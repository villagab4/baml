# This file is generated by the BAML compiler.
# Do not edit this file directly.
# Instead, edit the BAML files and recompile.
#
# BAML version: 0.0.1
# Generated Date: __DATE__
# Generated by: vbv

from ...._impl.deserializer import register_deserializer
from .cls_inputtype2 import InputType2
from pydantic import BaseModel


@register_deserializer()
class InputType(BaseModel):
    a: InputType2
    b: bool
