import * as anchor from "@anchor-lang/core";
import BN from "bn.js";
import { assert } from "chai";
import type { ConstraintsTrait } from "../target/types/constraints_trait";

describe("constraints-trait", () => {
  anchor.setProvider(anchor.AnchorProvider.env());
  const program = anchor.workspace
    .constraintsTrait as anchor.Program<ConstraintsTrait>;
  const provider = anchor.getProvider() as anchor.AnchorProvider;

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

  it("validates counter constraints", async () => {
    const counter = anchor.web3.Keypair.generate();

    await program.methods
      .initCounter(new BN(0))
      .accounts({
        counter: counter.publicKey,
        authority: provider.wallet.publicKey,
        systemProgram: anchor.web3.SystemProgram.programId,
      })
      .signers([counter])
      .rpc();

    await program.methods
      .increment()
      .accounts({
        counter: counter.publicKey,
        authority: provider.wallet.publicKey,
      })
      .rpc();

    const counterAccount = await program.account.counter.fetch(
      counter.publicKey
    );
    assert.strictEqual(counterAccount.count.toNumber(), 1);
    assert.ok(counterAccount.authority.equals(provider.wallet.publicKey));
  });
});
