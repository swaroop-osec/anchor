import * as anchor from "@anchor-lang/core";
import { AnchorError } from "@anchor-lang/core";
import BN from "bn.js";
import { assert } from "chai";
import type { ConstraintsTrait } from "../target/types/constraints_trait";

describe("constraints-trait", () => {
  anchor.setProvider(anchor.AnchorProvider.env());
  const program = anchor.workspace
    .constraintsTrait as anchor.Program<ConstraintsTrait>;
  const provider = anchor.getProvider() as anchor.AnchorProvider;
  let counterPda: anchor.web3.PublicKey;
  let counterBump: number;

  before(async () => {
    [counterPda, counterBump] = await anchor.web3.PublicKey.findProgramAddress(
      [Buffer.from("counter"), provider.wallet.publicKey.toBuffer()],
      program.programId
    );

    await program.methods
      .initCounter(new BN(0))
      .accounts({
        counter: counterPda,
        authority: provider.wallet.publicKey,
        systemProgram: anchor.web3.SystemProgram.programId,
      })
      .rpc();
  });

  it("runs constraints validation", async () => {
    await program.methods
      .noop()
      .accounts({
        authority: provider.wallet.publicKey,
        systemProgram: anchor.web3.SystemProgram.programId,
        rent: anchor.web3.SYSVAR_RENT_PUBKEY,
        selfProgram: program.programId,
      })
      .rpc();
  });

  it("fails manual validation for wrong program", async () => {
    try {
      await program.methods
        .noop()
        .accounts({
          authority: provider.wallet.publicKey,
          systemProgram: anchor.web3.SystemProgram.programId,
          rent: anchor.web3.SYSVAR_RENT_PUBKEY,
          selfProgram: anchor.web3.SystemProgram.programId,
        })
        .rpc();
      assert.ok(false);
    } catch (err) {
      assert.isTrue(err instanceof AnchorError);
      const anchorErr = err as AnchorError;
      assert.strictEqual(anchorErr.error.errorCode.code, "ConstraintAddress");
    }
  });

  it("validates counter constraints", async () => {
    await program.methods
      .increment()
      .accounts({
        counter: counterPda,
        authority: provider.wallet.publicKey,
      })
      .rpc();

    const counterAccount = await program.account.counter.fetch(counterPda);
    assert.strictEqual(counterAccount.count.toNumber(), 1);
    assert.strictEqual(counterAccount.bump, counterBump);
    assert.ok(counterAccount.authority.equals(provider.wallet.publicKey));
  });

  it("rejects wrong authority", async () => {
    const wrongAuthority = anchor.web3.Keypair.generate();

    try {
      await program.methods
        .increment()
        .accounts({
          counter: counterPda,
          authority: wrongAuthority.publicKey,
        })
        .signers([wrongAuthority])
        .rpc();
      assert.ok(false);
    } catch (err) {
      assert.isTrue(err instanceof AnchorError);
      const anchorErr = err as AnchorError;
      assert.strictEqual(anchorErr.error.errorCode.code, "ConstraintSeeds");
    }
  });
});
