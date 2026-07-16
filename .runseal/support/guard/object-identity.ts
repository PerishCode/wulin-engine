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
        !contract.includes("pub struct CanonicalObjectSnapshot") ||
        !contract.includes("pub publication_token: u64") ||
        !contract.includes("SourceReplaced") ||
        !contract.includes("OutsidePublishedWindow") ||
        contract.includes("pub struct CanonicalObject {\n    pub region:")
    ) fail("guard: canonical object identity/resolution contract diverged");

    const facade = await Deno.readTextFile(`${root}/crates/engine-runtime/src/runtime/mod.rs`);
    if (
        !facade.includes("pub fn resolve_canonical_object(") ||
        !facade.includes("identity: CanonicalObjectIdentity") ||
        !facade.includes("Result<CanonicalObjectResolution>") ||
        !facade.includes("pub fn canonical_object_snapshot(&self)") ||
        !facade.includes("Result<CanonicalObjectSnapshot>") ||
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
        !dispatch.includes("versioned-canonical-object-resolution-v2") ||
        !dispatch.includes("versioned-canonical-object-nearest-v2") ||
        dispatch.includes("exact-canonical-object-position-v1") ||
        dispatch.includes("exact-canonical-object-nearest-v1")
    ) fail("guard: workbench typed object-resolution schema diverged");

    const prototype = await Deno.readTextFile(`${root}/apps/prototype/src/observation.rs`);
    const prototypeMain = await Deno.readTextFile(`${root}/apps/prototype/src/main.rs`);
    const targetFields = prototype.match(
        /pub\(crate\) struct Target \{([\s\S]*?)\n\}/,
    )?.[1];
    if (
        !prototype.includes("pub(crate) struct Target") ||
        !prototype.includes("pub identity: CanonicalObjectIdentity") ||
        !prototype.includes("pub snapshot: CanonicalObjectSnapshot") ||
        !prototype.includes("OutsidePublishedWindow") ||
        !prototype.includes("SourceReplaced") ||
        !prototype.includes("validation_request") ||
        !prototype.includes("complete_validation") ||
        !targetFields ||
        (targetFields.match(/pub /g)?.length ?? 0) !== 3 ||
        !targetFields.includes("pub identity: CanonicalObjectIdentity") ||
        !targetFields.includes("pub snapshot: CanonicalObjectSnapshot") ||
        !targetFields.includes("pub availability: Availability") ||
        /CanonicalObject(?!Identity|Snapshot)|TerrainPosition|Presentation|Nearest/.test(
            targetFields,
        ) ||
        !prototypeMain.includes("if observation_policy.has_target()") ||
        !prototypeMain.includes("observation_policy.validation_request(snapshot)") ||
        !prototypeMain.includes("resolve_canonical_object(identity)") ||
        (prototypeMain.match(/query_nearest_canonical_object/g)?.length ?? 0) !== 1
    ) {
        fail("guard: prototype version-gated object target contract diverged");
    }
}
