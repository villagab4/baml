# This file is generated by the BAML compiler.
# Do not edit this file directly.
# Instead, edit the BAML files and recompile.
#
# BAML version: 0.0.1
# Generated Date: __DATE__
# Generated by: vbv

# ruff: noqa: E501,F401
# flake8: noqa: E501,F401
# pylint: disable=unused-import,line-too-long

from baml_core._impl.provider import llm_provider_factory
from os import environ


MyClient = llm_provider_factory(
    provider="openai",
    options=dict(
        model="gpt-35-turbo",
        api_key=environ['OPENAI_API_KEY'],
        my_custom_var="some string with spaces",
        temperature=0.3,
    ),
)