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

from ..clients.client_myclient import MyClient
from .fx_foobar2 import BAMLFooBar2
from baml_core._impl.deserializer import Deserializer


# Impl: SomeName
# Client: MyClient
# An implementation of .


__prompt_template = """\
does something
asdf goes here
a ;asdf;kljla
asflk;jasdf
{arg}

string\
"""


# We ignore the type here because baml does some type magic to make this work
# for inline SpecialForms like Optional, Union, List.
__deserializer = Deserializer[str](str)  # type: ignore


@BAMLFooBar2.register_impl("SomeName")
async def SomeName(arg: str, /) -> str:
    prompt = __prompt_template.format(arg=arg)
    response = await MyClient.run_prompt(prompt)
    return __deserializer.from_string(response.generated)