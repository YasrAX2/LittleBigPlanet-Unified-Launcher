import unittest
from scripts.check_npm_audit import BUILD_ONLY_ADVISORY, unexpected_findings


class AuditTests(unittest.TestCase):
    def test_exception_is_only_for_dev_dependencies(self):
        audit = {"vulnerabilities": {"picomatch": {"nodes": ["node_modules/picomatch"], "via": [{"name": "picomatch", "url": BUILD_ONLY_ADVISORY}]}}}
        self.assertEqual(unexpected_findings(audit, {"packages": {"node_modules/picomatch": {"dev": True}}}), [])
        self.assertEqual(unexpected_findings(audit, {"packages": {"node_modules/picomatch": {}}}), ["picomatch"])

    def test_new_advisory_fails(self):
        audit = {"vulnerabilities": {"picomatch": {"nodes": ["node_modules/picomatch"], "via": [{"name": "picomatch", "url": "https://example.com/new"}]}}}
        self.assertEqual(unexpected_findings(audit, {"packages": {"node_modules/picomatch": {"dev": True}}}), ["picomatch"])
