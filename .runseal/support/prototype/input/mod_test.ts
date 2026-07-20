import { preparePrototypeWindowAction } from "./mod.ts";
import { wideNullTerminated } from "./prepared.ts";

function assert(condition: boolean, message: string): void {
    if (!condition) throw new Error(message);
}

async function assertRejects(
    action: () => Promise<unknown>,
    expectedMessage: string,
): Promise<void> {
    try {
        await action();
    } catch (error) {
        assert(
            error instanceof Error && error.message.includes(expectedMessage),
            `unexpected error: ${String(error)}`,
        );
        return;
    }
    throw new Error(`expected rejection containing ${expectedMessage}`);
}

Deno.test("native input encodes null-terminated UTF-16 window names", () => {
    const encoded = wideNullTerminated("A武");
    assert(
        JSON.stringify([...encoded]) === JSON.stringify([0x41, 0x6b66, 0]),
        `unexpected encoding ${JSON.stringify([...encoded])}`,
    );
});

Deno.test("native input rejects invalid process identity before FFI work", async () => {
    await assertRejects(
        () => preparePrototypeWindowAction(0, [], true, "close"),
        "invalid process id 0",
    );
});

Deno.test("native input rejects action key-shape divergence before FFI work", async () => {
    await assertRejects(
        () => preparePrototypeWindowAction(1, [], true),
        "input action key shape diverged",
    );
});

Deno.test("native input rejects delayed atomic prefixes before FFI work", async () => {
    await assertRejects(
        () =>
            preparePrototypeWindowAction(
                1,
                [{ key: "W", virtualKey: 0x57, down: true }],
                true,
                "input",
                [1],
                0,
                true,
            ),
        "input action delay diverged",
    );
});
