"""Fail on npm advisories except one documented build-only, unpatched issue."""
import json
from pathlib import Path
import subprocess
import sys

BUILD_ONLY_ADVISORY = "https://github.com/advisories/GHSA-fc5q-39xc-vjpf"


def unexpected_findings(audit, lock):
    findings = []
    vulnerabilities = audit.get("vulnerabilities", {})
    for name, entry in vulnerabilities.items():
        nodes = entry.get("nodes", [])
        if not nodes or any(not lock["packages"].get(node, {}).get("dev") for node in nodes):
            findings.append(name)
            continue
        for via in entry.get("via", []):
            if isinstance(via, str):
                if via not in vulnerabilities:
                    findings.append(name)
            elif via.get("url") != BUILD_ONLY_ADVISORY or via.get("name") != "picomatch":
                findings.append(name)
    return sorted(set(findings))


if __name__ == "__main__":
    root = Path(__file__).resolve().parents[1]
    command = ["npm.cmd" if sys.platform == "win32" else "npm", "audit", "--json"]
    result = subprocess.run(command, cwd=root, capture_output=True, text=True)
    if result.returncode not in (0, 1):
        sys.exit(result.stderr or "npm audit could not complete")
    audit = json.loads(result.stdout)
    if "error" in audit or "vulnerabilities" not in audit:
        sys.exit("Incomplete npm audit response")
    unexpected = unexpected_findings(audit, json.loads((root / "package-lock.json").read_text()))
    if unexpected:
        sys.exit("Unaccepted npm advisories: " + ", ".join(unexpected))
    print("npm audit passed: no unaccepted advisories.")
    if audit["vulnerabilities"]:
        print("Known build-only picomatch ReDoS: fixed repository globs only; Node build tools are not shipped. " + BUILD_ONLY_ADVISORY)
