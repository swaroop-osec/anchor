import * as anchor from "@anchor-lang/core";
import { AnchorError, Program } from "@anchor-lang/core";
import { assert } from "chai";
import { AccountloaderRealloc } from "../target/types/accountloader_realloc";

describe("accountloader-realloc (triaged H-accountloader-realloc)", () => {
  anchor.setProvider(anchor.AnchorProvider.env());

  const program = anchor.workspace
    .AccountloaderRealloc as Program<AccountloaderRealloc>;
  const authority = (program.provider as any).wallet
    .payer as anchor.web3.Keypair;

  const DISCRIMINATOR_LEN = 8;

  let data: anchor.web3.PublicKey;

  before(async () => {
    [data] = anchor.web3.PublicKey.findProgramAddressSync(
      [Buffer.from("data")],
      program.programId
    );
  });

  it("initializes the zero-copy account at full size", async () => {
    await program.methods
      .initialize()
      .accounts({ authority: authority.publicKey, data })
      .rpc();

    const account = await program.account.data.fetch(data);
    assert.strictEqual(account.value.toString(), "42");
  });

  it("under-sized AccountLoader realloc aborts the tx with AccountDidNotDeserialize (3004) and does not brick", async () => {
    try {
      await program.methods
        .shrink(DISCRIMINATOR_LEN)
        .accounts({ authority: authority.publicKey, data })
        .rpc();
      assert.fail(
        "expected shrink of AccountLoader<Data> below 8 + size_of::<Data>() to abort the tx"
      );
    } catch (e) {
      assert.isTrue(e instanceof AnchorError, `unexpected error: ${e}`);
      const err = e as AnchorError;
      // 3003 = AccountDidNotDeserialize
      assert.strictEqual(
        err.error.errorCode.number,
        3003,
        `expected AccountDidNotDeserialize (3003), got ${err.error.errorCode.number} (${err.error.errorCode.code})`
      );
      assert.strictEqual(err.error.errorCode.code, "AccountDidNotDeserialize");
    }

    // Account body must be unchanged (Solana rolls back state on tx failure).
    const info = await program.provider.connection.getAccountInfo(data);
    assert.isNotNull(info, "account should still exist");
    assert.isAbove(
      info!.data.length,
      DISCRIMINATOR_LEN,
      "account body must NOT have been truncated"
    );

    // `load()` continues to work — value preserved.
    const value = await program.methods.read().accounts({ data }).view();
    assert.strictEqual(value.toString(), "42", "zero-copy value preserved");
  });

  it("accepts AccountLoader realloc at exactly DISCRIMINATOR + size_of::<Data>()", async () => {
    // The fix must NOT prevent same-size or larger reallocs. Same-size is the
    // common idempotent case (e.g. `realloc = 8 + size_of::<T>()`).
    const minSize = 8 + 8 + 64; // disc + value + padding
    await program.methods
      .shrink(minSize)
      .accounts({ authority: authority.publicKey, data })
      .rpc();

    const info = await program.provider.connection.getAccountInfo(data);
    assert.isNotNull(info);
    assert.strictEqual(info!.data.length, minSize);

    // Value still readable after a same-size realloc.
    const value = await program.methods.read().accounts({ data }).view();
    assert.strictEqual(value.toString(), "42");
  });
});
