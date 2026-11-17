"""Check the locked Windows dependency graph against OSV; send names/versions only."""
import concurrent.futures
import json
from pathlib import Path
import re
import subprocess
import sys
import tomllib
import urllib.request

root = Path(__file__).resolve().parents[1]
tree = subprocess.check_output([
    "cargo", "tree", "--manifest-path", str(root / "src-tauri/Cargo.toml"),
    "--locked", "--target", "x86_64-pc-windows-msvc", "--prefix", "none", "--format", "{p}",
], text=True)
used = set(re.findall(r"^([A-Za-z0-9_-]+) v([^\s]+)", tree, re.M))
locked = tomllib.loads((root / "src-tauri/Cargo.lock").read_text())["package"]
packages = [p for p in locked if p.get("source", "").startswith("registry+") and (p["name"], p["version"]) in used]


def query(batch):
    queries = [{"package": {"ecosystem": "crates.io", "name": p["name"]}, "version": p["version"]} for p in batch]
    findings = []
    while queries:
        request = urllib.request.Request("https://api.osv.dev/v1/querybatch", data=json.dumps({"queries": queries}).encode(), headers={"Content-Type": "application/json"})
        with urllib.request.urlopen(request, timeout=40) as response:
            results = json.load(response)["results"]
        if len(results) != len(queries):
            raise RuntimeError("Incomplete OSV response; audit not completed")
        next_queries = []
        for package, result in zip(queries, results):
            if result.get("vulns"):
                findings.append({"name": package["package"]["name"], "version": package["version"], "advisories": [v["id"] for v in result["vulns"]]})
            if result.get("next_page_token"):
                next_queries.append({**package, "page_token": result["next_page_token"]})
        queries = next_queries
    return findings


with concurrent.futures.ThreadPoolExecutor(max_workers=4) as pool:
    findings = [entry for group in pool.map(query, [packages[i:i+100] for i in range(0, len(packages), 100)]) for entry in group]
print(json.dumps({"target": "x86_64-pc-windows-msvc", "packagesChecked": len(packages), "advisories": findings}, indent=2))
sys.exit(bool(findings))
