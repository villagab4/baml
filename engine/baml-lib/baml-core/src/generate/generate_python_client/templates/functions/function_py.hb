{{> interface}}

class IBAML{{name}}(BaseBAMLFunction[{{return.0.type}}, {{return.0.type_partial}}]):
    def __init__(self) -> None:
        super().__init__(
            "{{name}}",
            I{{name}},
            [{{#each impls}}"{{this}}"{{#unless @last}}, {{/unless}}{{/each}}],
        )

    async def __call__(self, *args, **kwargs) -> {{return.0.type}}:
        {{#if has_impls}}
        return await self.get_impl("{{default_impl}}").run(*args, **kwargs)
        {{else}}
        raise NotImplemented("No impls defined")
        {{/if}}
    
    def stream(self, *args, **kwargs) -> AsyncStream[{{return.0.type}}, {{return.0.type_partial}}]:
        {{#if has_impls}}
        res = self.get_impl("{{default_impl}}").stream(*args, **kwargs)
        return res
        {{else}}
        raise NotImplemented("No impls defined")
        {{/if}}

BAML{{name}} = IBAML{{name}}()

__all__ = [ "BAML{{name}}" ]
