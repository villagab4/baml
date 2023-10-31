# This file is generated by the BAML compiler.
# Do not edit this file directly.
# Instead, edit the BAML files and recompile.
#
# BAML version: 0.0.1
# Generated Date: __DATE__
# Generated by: vbv

from .baml_types import IFooB, IFooBar, IFooBar2, IFunctionOne, IFunctionTwo
from .generated_baml_client import baml
from _pytest.fixtures import FixtureRequest


def FooBarImpl(request: FixtureRequest) -> IFooBar:
    """
    To use this fixture, add this to your test.
    Note the parameter name must match the name of this fixture.

    ```python
    @baml.FooBar.test
    async def test_logic(FooBarImpl: IFooBar) -> None:
        result = await FooBarImpl(args_here)
        ...
    ```

    See the docstring for baml.FooBar.test for more information.


    See pytest documentation for more information on fixtures:
    https://docs.pytest.org/en/latest/fixture.html
    """
    return baml.FooBar.get_impl(request.param).run

def FooBImpl(request: FixtureRequest) -> IFooB:
    """
    To use this fixture, add this to your test.
    Note the parameter name must match the name of this fixture.

    ```python
    @baml.FooB.test
    async def test_logic(FooBImpl: IFooB) -> None:
        result = await FooBImpl(args_here)
        ...
    ```

    See the docstring for baml.FooB.test for more information.


    See pytest documentation for more information on fixtures:
    https://docs.pytest.org/en/latest/fixture.html
    """
    return baml.FooB.get_impl(request.param).run

def FunctionOneImpl(request: FixtureRequest) -> IFunctionOne:
    """
    To use this fixture, add this to your test.
    Note the parameter name must match the name of this fixture.

    ```python
    @baml.FunctionOne.test
    async def test_logic(FunctionOneImpl: IFunctionOne) -> None:
        result = await FunctionOneImpl(args_here)
        ...
    ```

    See the docstring for baml.FunctionOne.test for more information.


    See pytest documentation for more information on fixtures:
    https://docs.pytest.org/en/latest/fixture.html
    """
    return baml.FunctionOne.get_impl(request.param).run

def FunctionTwoImpl(request: FixtureRequest) -> IFunctionTwo:
    """
    To use this fixture, add this to your test.
    Note the parameter name must match the name of this fixture.

    ```python
    @baml.FunctionTwo.test
    async def test_logic(FunctionTwoImpl: IFunctionTwo) -> None:
        result = await FunctionTwoImpl(args_here)
        ...
    ```

    See the docstring for baml.FunctionTwo.test for more information.


    See pytest documentation for more information on fixtures:
    https://docs.pytest.org/en/latest/fixture.html
    """
    return baml.FunctionTwo.get_impl(request.param).run

def FooBar2Impl(request: FixtureRequest) -> IFooBar2:
    """
    To use this fixture, add this to your test.
    Note the parameter name must match the name of this fixture.

    ```python
    @baml.FooBar2.test
    async def test_logic(FooBar2Impl: IFooBar2) -> None:
        result = await FooBar2Impl(args_here)
        ...
    ```

    See the docstring for baml.FooBar2.test for more information.


    See pytest documentation for more information on fixtures:
    https://docs.pytest.org/en/latest/fixture.html
    """
    return baml.FooBar2.get_impl(request.param).run

