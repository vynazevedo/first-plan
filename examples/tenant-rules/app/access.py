"""Minimal tenant-scoped access decision used by the verification example."""


def read_document(actor_tenant, document):
    if not actor_tenant or actor_tenant != document["tenant_id"]:
        raise PermissionError("tenant boundary")
    return document["body"]
