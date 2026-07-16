type Fail = (message: string) => never;

export async function requireQualifiedObjectIdentity(
    root: string,
    fail: Fail,
): Promise<void> {
    console.log("==> source-qualified canonical object identity");
    const contract = await Deno.readTextFile(
        `${root}/crates/engine-runtime/src/runtime/object_query.rs`,
    );
    if (
        !contract.includes("pub struct CanonicalObjectIdentity") ||
        !contract.includes("pub source_namespace: ObjectSourceNamespace") ||
        !contract.includes("pub identity: CanonicalObjectIdentity") ||
        contract.includes("pub struct CanonicalObject {\n    pub region:")
    ) fail("guard: canonical object identity is not solely source-qualified");

    const facade = await Deno.readTextFile(`${root}/crates/engine-runtime/src/runtime/mod.rs`);
    if (
        !facade.includes("identity: CanonicalObjectIdentity") ||
        facade.includes(".query_canonical_object(region, authored_local_id)")
    ) fail("guard: Runtime retains an unqualified canonical object lookup");

    const protocol = await Deno.readTextFile(
        `${root}/apps/workbench/src/inspect/protocol/objects.rs`,
    );
    const dispatch = await Deno.readTextFile(
        `${root}/apps/workbench/src/inspect/app/objects.rs`,
    );
    const acceptance = await Deno.readTextFile(`${root}/.runseal/support/object/query.ts`);
    if (
        !protocol.includes("source_namespace: String") ||
        !protocol.includes("decode_source_namespace") ||
        !acceptance.includes("unqualified canonical object query") ||
        dispatch.includes("exact-canonical-object-position-v1") ||
        dispatch.includes("exact-canonical-object-nearest-v1")
    ) fail("guard: workbench retains the unqualified object-query schema");
}
