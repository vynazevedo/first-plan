import unittest
from app.access import read_document


class TenantIsolation(unittest.TestCase):
    def test_own_tenant_can_read(self):
        self.assertEqual(read_document("alpha", {"tenant_id": "alpha", "body": "private"}), "private")

    def test_other_tenant_cannot_read(self):
        with self.assertRaises(PermissionError):
            read_document("beta", {"tenant_id": "alpha", "body": "private"})

    def test_missing_tenant_cannot_read(self):
        with self.assertRaises(PermissionError):
            read_document(None, {"tenant_id": "alpha", "body": "private"})
