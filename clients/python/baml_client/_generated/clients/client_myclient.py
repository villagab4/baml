# This file is generated by the BAML compiler.
# Do not edit this file directly.
# Instead, edit the BAML files and recompile.
#
# BAML version: 0.0.1
# Generated Date: __DATE__
# Generated by: vbv

from ..._impl.provider import llm_provider_factory
from os import environ


MyClient = llm_provider_factory(
    provider="openai",
    options=dict(
        my_custom_var="some string with spaces",
        api_key=environ['OPENAI_API_KEY'],
        temperature=0.3,
        model="gpt-35-turbo",
    ),
)
