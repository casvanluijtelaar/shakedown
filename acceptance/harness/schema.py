"""A small JSON Schema 2020-12 validator for the keywords result.schema.json and the MCP tools' schemas use.

It knows only the keywords listed in KNOWN. A schema that uses any other keyword is refused,
so a new keyword in the contract shows up as a harness error instead of being skipped.
"""

import re

KNOWN = {
    "$schema", "$id", "$comment", "$defs", "$ref", "title", "description",
    "type", "const", "enum", "pattern", "minLength", "minimum", "maximum", "exclusiveMinimum",
    "properties", "required", "additionalProperties", "propertyNames",
    "items", "minItems", "oneOf", "anyOf", "allOf", "if", "then", "else",
}

_TYPES = {
    "object": lambda v: isinstance(v, dict),
    "array": lambda v: isinstance(v, list),
    "string": lambda v: isinstance(v, str),
    "boolean": lambda v: isinstance(v, bool),
    "null": lambda v: v is None,
    "integer": lambda v: (isinstance(v, int) and not isinstance(v, bool))
    or (isinstance(v, float) and v.is_integer()),
    "number": lambda v: isinstance(v, (int, float)) and not isinstance(v, bool),
}


class SchemaError(Exception):
    """The schema itself uses something this validator does not know."""


def _equal(a, b):
    if isinstance(a, bool) or isinstance(b, bool):
        return type(a) is type(b) and a == b
    return a == b


def validate(instance, schema):
    """Returns a list of error strings; empty when the instance is valid."""
    errors = []
    _check(instance, schema, schema, "$", errors)
    return errors


def _resolve(root, ref):
    if not ref.startswith("#/"):
        raise SchemaError(f"only local $ref is supported, got {ref}")
    node = root
    for part in ref[2:].split("/"):
        node = node[part]
    return node


def _check(value, schema, root, path, errors):
    if schema is True or schema == {}:
        return
    if schema is False:
        errors.append(f"{path}: no value is allowed here")
        return
    unknown = set(schema) - KNOWN
    if unknown:
        raise SchemaError(f"unknown schema keywords at {path}: {sorted(unknown)}")

    if "$ref" in schema:
        _check(value, _resolve(root, schema["$ref"]), root, path, errors)

    if "type" in schema:
        types = schema["type"] if isinstance(schema["type"], list) else [schema["type"]]
        if not any(_TYPES[t](value) for t in types):
            errors.append(f"{path}: expected type {types}, got {type(value).__name__} {value!r:.80}")
            return

    if "const" in schema and not _equal(value, schema["const"]):
        errors.append(f"{path}: expected {schema['const']!r}, got {value!r:.80}")
    if "enum" in schema and not any(_equal(value, e) for e in schema["enum"]):
        errors.append(f"{path}: {value!r:.80} is not one of {schema['enum']}")

    if isinstance(value, str):
        if "pattern" in schema and not re.search(schema["pattern"], value):
            errors.append(f"{path}: {value!r} does not match {schema['pattern']}")
        if "minLength" in schema and len(value) < schema["minLength"]:
            errors.append(f"{path}: shorter than {schema['minLength']}")

    if _TYPES["number"](value) and "minimum" in schema and value < schema["minimum"]:
        errors.append(f"{path}: {value} is below {schema['minimum']}")
    if _TYPES["number"](value) and "maximum" in schema and value > schema["maximum"]:
        errors.append(f"{path}: {value} is above {schema['maximum']}")
    if _TYPES["number"](value) and "exclusiveMinimum" in schema and value <= schema["exclusiveMinimum"]:
        errors.append(f"{path}: {value} is not above {schema['exclusiveMinimum']}")

    if isinstance(value, dict):
        props = schema.get("properties", {})
        for key in schema.get("required", []):
            if key not in value:
                errors.append(f"{path}: missing required key {key!r}")
        for key, item in value.items():
            if "propertyNames" in schema:
                _check(key, schema["propertyNames"], root, f"{path}.{key}(name)", errors)
            if key in props:
                _check(item, props[key], root, f"{path}.{key}", errors)
            elif "additionalProperties" in schema:
                _check(item, schema["additionalProperties"], root, f"{path}.{key}", errors)

    if isinstance(value, list):
        if "minItems" in schema and len(value) < schema["minItems"]:
            errors.append(f"{path}: fewer than {schema['minItems']} items")
        if "items" in schema:
            for i, item in enumerate(value):
                _check(item, schema["items"], root, f"{path}[{i}]", errors)

    if "oneOf" in schema:
        passing = sum(1 for s in schema["oneOf"] if not _sub(value, s, root, path))
        if passing != 1:
            errors.append(f"{path}: matches {passing} of the oneOf branches, expected exactly 1")

    if "anyOf" in schema and not any(not _sub(value, s, root, path) for s in schema["anyOf"]):
        errors.append(f"{path}: matches none of the anyOf branches")

    for sub in schema.get("allOf", []):
        _check(value, sub, root, path, errors)

    if "if" in schema:
        branch = "then" if not _sub(value, schema["if"], root, path) else "else"
        if branch in schema:
            _check(value, schema[branch], root, path, errors)


def _sub(value, schema, root, path):
    errs = []
    _check(value, schema, root, path, errs)
    return errs
