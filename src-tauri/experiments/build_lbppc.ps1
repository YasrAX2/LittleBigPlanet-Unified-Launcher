param([Parameter(Mandatory=$true)][string]$Workspace,[Parameter(Mandatory=$true)][string]$Destination)
$ErrorActionPreference='Stop'
$ProgressPreference='SilentlyContinue'
[Net.ServicePointManager]::SecurityProtocol=[Net.SecurityProtocolType]::Tls12
function Step([string]$Message){Write-Output "PODBUILD: $Message"}
function Checked([string]$File,[string[]]$Arguments){ & $File @Arguments; if($LASTEXITCODE -ne 0){throw "Build command failed ($LASTEXITCODE). See the build log."} }
function NoLinks([string]$Path){
 $checked=[IO.Path]::GetFullPath($Path)
 while($checked){
  $item=Get-Item -LiteralPath $checked -Force -ErrorAction SilentlyContinue
  if($item -and ($item.Attributes -band [IO.FileAttributes]::ReparsePoint)){throw 'Build paths cannot contain links or junctions.'}
  $parent=Split-Path $checked -Parent
  if($parent -eq $checked){break};$checked=$parent
 }
}
$Workspace=[IO.Path]::GetFullPath($Workspace)
$Destination=[IO.Path]::GetFullPath($Destination)
if($Workspace -match '[^\x21-\x7e]'){throw 'Choose a short build folder with ASCII characters and no spaces.'}
NoLinks $Workspace; NoLinks $Destination
New-Item -ItemType Directory -Force -Path $Workspace | Out-Null
$drive=Get-PSDrive -Name ([IO.Path]::GetPathRoot($Workspace).Substring(0,1))
if($drive.Free -lt 20GB){throw 'Source builds need at least 20 GB of free space.'}
$tools=Join-Path $Workspace 'tools'
$msys=Join-Path $tools 'msys64'
$bash=Join-Path $msys 'usr\bin\bash.exe'
NoLinks $tools;NoLinks $bash
if(-not(Test-Path -LiteralPath $bash)){
 Step 'Downloading verified MSYS2 build tools'
 New-Item -ItemType Directory -Force -Path $tools | Out-Null
 $archive=Join-Path $tools 'msys2-base-20260927.sfx.exe'
 $expected='ad336cccfda47758b5e15cda993fbba421115cb0b126697daef1ee4dfe37209f'
 if(-not(Test-Path -LiteralPath $archive) -or (Get-FileHash -LiteralPath $archive -Algorithm SHA256).Hash.ToLower() -ne $expected){
  Invoke-WebRequest -UseBasicParsing -Uri 'https://github.com/msys2/msys2-installer/releases/download/2026-09-27/msys2-base-x86_64-20260927.sfx.exe' -OutFile $archive
 }
 if((Get-FileHash -LiteralPath $archive -Algorithm SHA256).Hash.ToLower() -ne $expected){throw 'MSYS2 checksum mismatch; refusing to run it.'}
 Checked $archive @('-y',('-o'+$tools))
}
# Use the primary repository directly when a redirect-selected mirror is unavailable.
[IO.File]::WriteAllText((Join-Path $msys 'etc\pacman.d\mirrorlist.mingw'),'Server = https://repo.msys2.org/mingw/$repo/'+"`n",(New-Object Text.UTF8Encoding($false)))
Step 'Preparing the MSYS2 environment'
$env:MSYSTEM='CLANG64';$env:CHERE_INVOKING='1';$env:MSYS2_PATH_TYPE='inherit'
Checked $bash @('--login','-c','true')
Step 'Updating signed compiler packages (first build can take a while)'
# Core-runtime updates can terminate their first shell; a fresh shell completes the update.
& $bash --login -c 'pacman -Syu --noconfirm'
Checked $bash @('--login','-c','pacman -Syu --noconfirm')
function WindowsPython {
 $candidates=@((Get-Command python.exe -ErrorAction SilentlyContinue).Source)
 $candidates+=@(Get-ChildItem -Path "$env:LOCALAPPDATA\Programs\Python\Python3*\python.exe" -ErrorAction SilentlyContinue | Sort-Object LastWriteTime -Descending | Select-Object -ExpandProperty FullName)
 foreach($candidate in ($candidates | Select-Object -Unique)){
  if($candidate -and $candidate -notlike '*WindowsApps*'){
   & $candidate -c 'import sys; raise SystemExit(sys.version_info < (3,10))'
   if($LASTEXITCODE -eq 0){return $candidate}
  }
 }
}
$python=WindowsPython
if(-not $python){
 Step 'Installing Windows Python for official LBPPC packaging'
 $winget=(Get-Command winget.exe -ErrorAction SilentlyContinue).Source
 if(-not $winget){throw 'Windows Python 3.10+ is missing and winget is unavailable. Install Python from python.org, then retry.'}
 Checked $winget @('install','--id','Python.Python.3.13','--exact','--scope','user','--silent','--accept-package-agreements','--accept-source-agreements')
 $python=WindowsPython
}
if(-not $python){throw 'A supported Windows Python installation could not be located.'}
$source=Join-Path $Workspace 'lbppc-v0.1.5'
$env:POD_SOURCE_WIN=$source;$env:POD_PYTHON_WIN=$python
$env:CMAKE_BUILD_PARALLEL_LEVEL='2'
$driver=Join-Path $Workspace 'build-driver.sh'
$driverText=@'
#!/usr/bin/env bash
set -euo pipefail
echo 'PODBUILD: Installing signed LBPPC compiler dependencies'
pacman -S --needed --noconfirm git patch \
 mingw-w64-clang-x86_64-{toolchain,cmake,ninja,pkgconf,python} \
 mingw-w64-clang-x86_64-{vulkan-headers,vulkan-loader,vulkan-memory-allocator,glslang} \
 mingw-w64-clang-x86_64-{sdl3,fmt,boost,robin-map,xxhash,ffmpeg,zydis} \
 mingw-w64-clang-x86_64-{spirv-tools,spirv-headers,spirv-cross} \
 mingw-w64-clang-x86_64-{protobuf,openssl,zlib,miniupnpc,nlohmann-json}
source_dir=$(cygpath -u "$POD_SOURCE_WIN")
if [[ ! -d "$source_dir/.git" ]]; then
 echo 'PODBUILD: Downloading official LBPPC v0.1.5 source'
 git -c core.longpaths=true clone --branch v0.1.5 --depth 1 https://github.com/emurphy04/LBPPC.git "$source_dir"
fi
cd "$source_dir"
[[ $(git rev-parse HEAD) == 3a3c2111bb0a137f1c564ccd2f16e357393ac059 ]] || { echo 'Unexpected source revision'; exit 1; }
git diff --quiet --ignore-submodules=all HEAD -- || { echo 'Source files changed; choose a fresh build folder'; exit 1; }
git -c core.longpaths=true submodule update --init --recursive
echo 'PODBUILD: Compiling LBPPC (two compiler jobs)'
export CMAKE_BUILD_PARALLEL_LEVEL=2
# The official script calls ninja directly, so bound its parallelism as well.
ninja() { command ninja -j2 "$@"; }
export -f ninja
bash build.sh
echo 'PODBUILD: Packaging the source build'
# The release packaging script expects a runtime-created file; use its tracked default on a fresh checkout.
[[ -f input.ini ]] || cp packaging/input.ini input.ini
export WINPYTHON=$(cygpath -u "$POD_PYTHON_WIN")
bash packaging/windows/package.sh
'@
[IO.File]::WriteAllText($driver,$driverText.Replace("`r`n","`n"),(New-Object Text.UTF8Encoding($false)))
$env:MSYS2_ROOT=(& (Join-Path $msys 'usr\bin\cygpath.exe') -u $msys).Trim()
$driverMsys=(& (Join-Path $msys 'usr\bin\cygpath.exe') -u $driver).Trim()
NoLinks (Join-Path $source 'out\stage')
Checked $bash @('--login',$driverMsys)
$stage=Join-Path $source 'out\stage\LBPPC'
if(-not(Test-Path -LiteralPath (Join-Path $stage 'LBPPC.exe')) -or -not(Test-Path -LiteralPath (Join-Path $stage 'bin\LBPPC-Game.exe'))){throw 'The build did not produce a complete Windows package.'}
if(Test-Path -LiteralPath $Destination){throw 'The destination already exists; refusing to replace an engine.'}
NoLinks $stage;NoLinks (Split-Path $Destination)
Get-ChildItem -LiteralPath $stage -Recurse -Force | ForEach-Object {if($_.Attributes -band [IO.FileAttributes]::ReparsePoint){throw 'Build output contains a link; refusing to copy it.'}}
New-Item -ItemType Directory -Force -Path $Destination | Out-Null
Get-ChildItem -LiteralPath $stage -Force | Copy-Item -Destination $Destination -Recurse -Force
Step 'Verifying compiled package'
