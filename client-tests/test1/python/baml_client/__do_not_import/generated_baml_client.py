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

from .clients.client_azure_default import AZURE_DEFAULT
from .clients.client_azure_gpt4 import AZURE_GPT4
from .clients.client_azure_yes_no import AZURE_YES_NO
from .clients.client_large_response import LARGE_RESPONSE
from .clients.client_resilientgpt4 import ResilientGPT4
from .functions.fx_blah import BAMLBlah
from .functions.fx_classifytool import BAMLClassifyTool
from .functions.fx_maybepolishtext import BAMLMaybePolishText
from .functions.fx_messagesimplifier import BAMLMessageSimplifier
from .functions.fx_textpolisher import BAMLTextPolisher


from baml_lib import baml_init 
class BAMLClient:
    Blah = BAMLBlah
    ClassifyTool = BAMLClassifyTool
    MaybePolishText = BAMLMaybePolishText
    MessageSimplifier = BAMLMessageSimplifier
    TextPolisher = BAMLTextPolisher
    AZURE_DEFAULT = AZURE_DEFAULT
    AZURE_GPT4 = AZURE_GPT4
    AZURE_YES_NO = AZURE_YES_NO
    LARGE_RESPONSE = LARGE_RESPONSE
    ResilientGPT4 = ResilientGPT4

    def __init__(self):
        baml_init()

baml = BAMLClient()
