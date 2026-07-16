type Fail = (message: string) => never;

export async function requireTypedObjectResolution(
    root: string,
    fail: Fail,
): Promise<void> {
    console.log("==> typed source-qualified canonical object resolution");
    const contract = await Deno.readTextFile(
        `${root}/crates/engine-runtime/src/runtime/object_query.rs`,
    );
    if (
        !contract.includes("pub struct CanonicalObjectIdentity") ||
        !contract.includes("pub source_namespace: ObjectSourceNamespace") ||
        !contract.includes("pub identity: CanonicalObjectIdentity") ||
        !contract.includes("pub enum CanonicalObjectResolution") ||
        !contract.includes("SourceReplaced") ||
        !contract.includes("OutsidePublishedWindow") ||
        contract.includes("pub struct CanonicalObject {\n    pub region:")
    ) fail("guard: canonical object identity/resolution contract diverged");

    const facade = await Deno.readTextFile(`${root}/crates/engine-runtime/src/runtime/mod.rs`);
    if (
        !facade.includes("pub fn resolve_canonical_object(") ||
        !facade.includes("identity: CanonicalObjectIdentity") ||
        !facade.includes("Result<CanonicalObjectResolution>") ||
        facade.includes("query_canonical_object")
    ) fail("guard: Runtime does not expose only the typed qualified resolver");

    const protocol = await Deno.readTextFile(
        `${root}/apps/workbench/src/inspect/protocol/objects.rs`,
    );
    const dispatch = await Deno.readTextFile(
        `${root}/apps/workbench/src/inspect/app/objects.rs`,
    );
    const routing = await Deno.readTextFile(`${root}/apps/workbench/src/inspect/protocol.rs`);
    const acceptance = await Deno.readTextFile(`${root}/.runseal/support/object/query.ts`);
    if (
        !protocol.includes("source_namespace: String") ||
        !protocol.includes("decode_source_namespace") ||
        !routing.includes('"canonical.objects.resolve" => objects::resolve(payload)') ||
        routing.includes('"canonical.objects.query" =>') ||
        !acceptance.includes("unqualified canonical object resolution") ||
        !acceptance.includes("retired canonical.objects.query verb remains live") ||
        !dispatch.includes("typed-canonical-object-resolution-v1") ||
        dispatch.includes("exact-canonical-object-position-v1") ||
        dispatch.includes("exact-canonical-object-nearest-v1")
    ) fail("guard: workbench typed object-resolution schema diverged");

    const prototype = await Deno.readTextFile(`${root}/apps/prototype/src/observation.rs`);
    if (
        prototype.includes("resolve_canonical_object") ||
        prototype.includes("CanonicalObjectIdentity")
    ) {
        fail("guard: prototype retains or recursively resolves an object target before admission");
    }
}
