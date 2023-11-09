# This file is generated by the BAML compiler.
# Do not edit this file directly.
# Instead, edit the BAML files and recompile.
#
# BAML version: 0.1.1-canary.8
# Generated Date: __DATE__
# Generated by: __USER__

# ruff: noqa: E501,F401
# flake8: noqa: E501,F401
# pylint: disable=unused-import,line-too-long
# fmt: off

from ..clients.client_resilientgpt4 import ResilientGPT4
from ..functions.fx_blah import BAMLBlah
from baml_lib._impl.deserializer import Deserializer


# Impl: v1
# Client: ResilientGPT4
# An implementation of .


__prompt_template = """\
hello there {arg}\
"""

__input_replacers = {
    "{arg}"
}


# We ignore the type here because baml does some type magic to make this work
# for inline SpecialForms like Optional, Union, List.
__deserializer = Deserializer[str](str)  # type: ignore

@BAMLBlah.register_impl("v1")
async def v1(arg: str, /) -> str:
    response = await ResilientGPT4.run_prompt_template(template=__prompt_template, replacers=__input_replacers, params=dict(arg=arg))
    return __deserializer.from_string(response.generated)
