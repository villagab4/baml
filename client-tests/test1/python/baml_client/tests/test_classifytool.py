# This file is generated by the BAML compiler.
# Do not edit this file directly.
# Instead, edit the BAML files and recompile.

# ruff: noqa: E501,F401
# flake8: noqa: E501,F401
# pylint: disable=unused-import,line-too-long
# fmt: off

from ..__do_not_import.generated_baml_client import baml
from ..baml_types import ClassifyResponse, IClassifyTool, Tool
from baml_lib._impl.deserializer import Deserializer
from json import dumps
from typing import Any


@baml.ClassifyTool.test
async def test_fun_indigo(ClassifyToolImpl: IClassifyTool):
    def to_str(item: Any) -> str:
        if isinstance(item, str):
            return item
        return dumps(item)

    case = {"query": "sesefsfsefsefsefsefsefsefsfsefsefsefsefsefsefsefsefsefsef", "context": None, }
    deserializer_query = Deserializer[str](str) # type: ignore
    query = deserializer_query.from_string(to_str(case["query"]))
    deserializer_context = Deserializer[str](str) # type: ignore
    context = deserializer_context.from_string(to_str(case["context"]))
    await ClassifyToolImpl(
        query=query,
        context=context
    )


@baml.ClassifyTool.test
async def test_particular_tan(ClassifyToolImpl: IClassifyTool):
    def to_str(item: Any) -> str:
        if isinstance(item, str):
            return item
        return dumps(item)

    case = {"query": "sefesfsfsefsefsefesfsfsefsefsefesfsfsefsefsefesfsfsefsefsefesfsfsefsefsefesfsfsefsefsefesfsfsefsefsefesfsfsefsefsefesfsfsefsefsefesfsfsefsefsefesfsfsefsefsefesfsfsefsefsefesfsfsefsefsefesfsfsefsefsefesfsfsefsefsefesfsfsefsefsefesfsfsefsefsefesfsfsefsefsefesfsfsefsefsefesfsfsefsefsefesfsfsefsefsefesfsfsefsefsefesfsfsefsefsefesfsfsefsefsefesfsfsefsef", "context": "sefesfsfsefsefsefesfsfsefsefsefesfsfsefsefsefesfsfsefsefsefesfsfsefsefsefesfsfsefsef", }
    deserializer_query = Deserializer[str](str) # type: ignore
    query = deserializer_query.from_string(to_str(case["query"]))
    deserializer_context = Deserializer[str](str) # type: ignore
    context = deserializer_context.from_string(to_str(case["context"]))
    await ClassifyToolImpl(
        query=query,
        context=context
    )


