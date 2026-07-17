type Fail = (message: string) => never;

export async function requireTypedObjectResolution(
    root: string,
    fail: Fail,
): Promise<void> {
    console.log("==> typed source-qualified canonical object lifecycle");
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
        !contract.includes("pub struct CanonicalObjectProximity") ||
        !contract.includes("pub fn proximity_from(") ||
        !contract.includes("pub struct ObjectTargetFeedback") ||
        !contract.includes("pub enum ObjectTargetFeedbackKind") ||
        !contract.includes("Selected") ||
        !contract.includes("Activated") ||
        !contract.includes("Rejected") ||
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
    const feedback = await Deno.readTextFile(`${root}/.runseal/support/object/feedback.ts`);
    const nearest = await Deno.readTextFile(
        `${root}/crates/engine-runtime/src/rendering/async_resident/renderer/query.rs`,
    );
    if (
        !protocol.includes("source_namespace: String") ||
        !protocol.includes("decode_source_namespace") ||
        !routing.includes('"canonical.objects.resolve" => objects::resolve(payload)') ||
        routing.includes('"canonical.objects.query" =>') ||
        !acceptance.includes("unqualified canonical object resolution") ||
        !acceptance.includes("retired canonical.objects.query verb remains live") ||
        !dispatch.includes("versioned-canonical-object-resolution-v2") ||
        !dispatch.includes("versioned-canonical-object-nearest-v3") ||
        !routing.includes('"canonical.objects.target.set" => objects::target(payload)') ||
        !routing.includes('"canonical.objects.target.clear" =>') ||
        !routing.includes('"canonical.objects.suppression.set" => objects::suppression(payload)') ||
        !routing.includes('"canonical.objects.suppression.clear" =>') ||
        !feedback.includes("targetedPixels") ||
        !feedback.includes("visibleObjectTarget") ||
        !feedback.includes('"activated" | "rejected" | "selected"') ||
        !feedback.includes("invalidObjectFeedbackGate") ||
        !feedback.includes("objectSuppressionLifecycle") ||
        !feedback.includes("assertUnprojectedSuppression") ||
        !nearest.includes("excluded_identity: Option<CanonicalObjectIdentity>") ||
        !nearest.includes("if Some(object.identity) == excluded_identity") ||
        !nearest.includes("object.proximity_from(origin, max_distance_q9)?") ||
        dispatch.includes("exact-canonical-object-position-v1") ||
        dispatch.includes("exact-canonical-object-nearest-v1")
    ) fail("guard: workbench typed object-resolution schema diverged");

    const prototype = await Deno.readTextFile(
        `${root}/apps/prototype/src/object/observation.rs`,
    );
    const interaction = await Deno.readTextFile(
        `${root}/apps/prototype/src/object/interaction.rs`,
    );
    const prototypeMain = await Deno.readTextFile(`${root}/apps/prototype/src/main.rs`);
    const prototypeSession = await Deno.readTextFile(`${root}/apps/prototype/src/session.rs`);
    const runtimeFrame = await Deno.readTextFile(
        `${root}/crates/engine-runtime/src/rendering/renderer/frame.rs`,
    );
    const skeletalShader = await Deno.readTextFile(
        `${root}/crates/engine-runtime/shaders/skeletal_scene.hlsl`,
    );
    const surfaceShader = await Deno.readTextFile(
        `${root}/crates/engine-runtime/shaders/surface_resolve.hlsl`,
    );
    const runtime = await Deno.readTextFile(`${root}/crates/engine-runtime/src/runtime/mod.rs`);
    const renderer = await Deno.readTextFile(
        `${root}/crates/engine-runtime/src/rendering/renderer/mod.rs`,
    );
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
        !prototypeMain.includes("let object_target = observation_policy.target_identity()") ||
        !prototypeSession.includes('"frameFeedback"') ||
        !prototypeMain.includes("const ACTIVATE_OBJECT: u8 = 0x0D") ||
        !prototypeMain.includes(".prepare_after_advance(") ||
        !prototypeMain.includes(".complete_frame(") ||
        !prototypeSession.includes('"object_interaction_driver"') ||
        !interaction.includes("pub(crate) const OBJECT_ACTION_RADIUS_Q9: u32 = 512") ||
        !interaction.includes("pub(crate) const ACKNOWLEDGEMENT_FRAME_COUNT: u32 = 12") ||
        !interaction.includes("object.proximity_from(origin, OBJECT_ACTION_RADIUS_Q9)?") ||
        !interaction.includes("kind: ObjectTargetFeedbackKind::Activated") ||
        !interaction.includes("CapacityExhausted") ||
        !interaction.includes("pub(crate) const fn nearest_exclusion") ||
        !interaction.includes("pub(crate) const fn frame_suppression") ||
        !interaction.includes("pub(crate) fn observe_source") ||
        !runtime.includes("pub object_target_feedback: Option<ObjectTargetFeedback>") ||
        !runtime.includes("pub object_suppression: Option<CanonicalObjectIdentity>") ||
        !renderer.includes("pub object_target_feedback: Option<ObjectTargetFeedback>") ||
        !renderer.includes("pub object_suppression: Option<CanonicalObjectIdentity>") ||
        !prototypeMain.includes("resolve_canonical_object(identity)") ||
        !prototypeMain.includes("interaction_policy.nearest_exclusion()") ||
        !prototypeMain.includes("interaction_policy.frame_suppression()") ||
        !prototypeMain.includes("observation_policy.clear_target(identity)") ||
        !runtimeFrame.includes("projected_object_target_feedback") ||
        !runtimeFrame.includes("object_target_feedback") ||
        !runtimeFrame.includes("project_suppression(") ||
        !runtimeFrame.includes("projected_object_suppression") ||
        !skeletalShader.includes("stable_identity_high = local_id;") ||
        !skeletalShader.includes("(suppression & 0x80000000u) != 0u") ||
        !skeletalShader.includes("group_id.x == ((suppression >> 10u) & 31u)") ||
        !skeletalShader.includes("local_id == (suppression & 1023u)") ||
        !surfaceShader.includes("visible.semantic_region == surface_animation.z") ||
        !surfaceShader.includes("visible.stable_identity_high == surface_animation.w") ||
        !surfaceShader.includes("surface_animation.y == 1u") ||
        !surfaceShader.includes("float3(0.12, 1.0, 0.32)") ||
        !surfaceShader.includes("surface_stats.InterlockedAdd(20, group_targeted") ||
        (prototypeMain.match(/query_nearest_canonical_object/g)?.length ?? 0) !== 1
    ) {
        fail("guard: prototype version-gated object target contract diverged");
    }
}
