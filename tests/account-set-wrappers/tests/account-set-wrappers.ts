import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { Keypair, PublicKey, SystemProgram } from "@solana/web3.js";
import { assert } from "chai";
import BN from "bn.js";
import type { AccountSetWrappers } from "../target/types/account_set_wrappers";

describe("account-set-wrappers", () => {
  const provider = anchor.AnchorProvider.env();
  anchor.setProvider(provider);

  const program = anchor.workspace.AccountSetWrappers as Program<AccountSetWrappers>;

  let dataKeypair: Keypair;
  let pdaAddress: PublicKey;
  let pdaBump: number;

  before(async () => {
    // Generate keypairs for test accounts
    dataKeypair = Keypair.generate();

    // Find the PDA for seeded tests
    [pdaAddress, pdaBump] = await PublicKey.findProgramAddress(
      [Buffer.from("test_data")],
      program.programId
    );
  });

  describe("Initialization", () => {
    it("initializes a regular data account", async () => {
      await program.methods
        .initialize(new BN(100))
        .accounts({
          data: dataKeypair.publicKey,
          authority: provider.wallet.publicKey,
          systemProgram: SystemProgram.programId,
        })
        .signers([dataKeypair])
        .rpc();

      const account = await program.account.testData.fetch(dataKeypair.publicKey);
      assert.equal(account.value.toNumber(), 100);
      assert.ok(account.authority.equals(provider.wallet.publicKey));
    });

    it("initializes a PDA account", async () => {
      await program.methods
        .initPda(new BN(200))
        .accounts({
          data: pdaAddress,
          authority: provider.wallet.publicKey,
          systemProgram: SystemProgram.programId,
        })
        .rpc();

      const account = await program.account.testData.fetch(pdaAddress);
      assert.equal(account.value.toNumber(), 200);
      assert.equal(account.bump, pdaBump);
      assert.ok(account.authority.equals(provider.wallet.publicKey));
    });

  });

  // =========================================================================
  // Tests for Mut<T> and Seeded<T, S> wrapper types (supported in derive macro)
  // =========================================================================

  describe("Mut<T> as account type", () => {
    it("Mut<Account> validates writable and allows modification", async () => {
      await program.methods
        .testMutAsType(new BN(150))
        .accounts({
          data: dataKeypair.publicKey,
          authority: provider.wallet.publicKey,
        })
        .rpc();

      const account = await program.account.testData.fetch(dataKeypair.publicKey);
      assert.equal(account.value.toNumber(), 150);
    });
  });

  describe("Seeded<T, S> as account type", () => {
    it("Seeded<Account, Seeds> validates PDA and captures bump", async () => {
      await program.methods
        .testSeededAsType()
        .accounts({
          data: pdaAddress,
          authority: provider.wallet.publicKey,
        })
        .rpc();
    });
  });

  describe("Composed wrapper types", () => {
    it("Mut<Seeded<Account, Seeds>> validates writable AND PDA", async () => {
      await program.methods
        .testMutSeededAsType(new BN(700))
        .accounts({
          data: pdaAddress,
          authority: provider.wallet.publicKey,
        })
        .rpc();

      const account = await program.account.testData.fetch(pdaAddress);
      assert.equal(account.value.toNumber(), 700);
    });
  });

  describe("SingleAccountSet trait", () => {
    it("SingleAccountSet trait methods work through Mut<Account> type", async () => {
      await program.methods
        .testSingleAccountSetTrait()
        .accounts({
          data: dataKeypair.publicKey,
          authority: provider.wallet.publicKey,
        })
        .rpc();
    });
  });

  // =========================================================================
  // Tests for NEW wrapper types (Owned, Executable, HasOne, Close, Realloc)
  // HasOne auto-validates in Constraints; some others still use manual checks in handlers.
  // =========================================================================

  describe("Owned<T, P> wrapper", () => {
    it("validates account owner matches program", async () => {
      await program.methods
        .testOwnedWrapper()
        .accounts({
          data: dataKeypair.publicKey,
          authority: provider.wallet.publicKey,
        })
        .rpc();
    });
  });

  describe("Executable<T> wrapper", () => {
    it("validates account is executable (program)", async () => {
      // Use the system program as an example of an executable account
      await program.methods
        .testExecutableWrapper()
        .accounts({
          programAccount: SystemProgram.programId,
          authority: provider.wallet.publicKey,
        })
        .rpc();
    });
  });

  describe("HasOne<T, Target> wrapper", () => {
    it("validates account relationship (authority matches)", async () => {
      await program.methods
        .testHasOneWrapper()
        .accounts({
          data: dataKeypair.publicKey,
          authority: provider.wallet.publicKey,
        })
        .rpc();
    });

    it("fails when authority doesn't match", async () => {
      const wrongAuthority = Keypair.generate();

      try {
        await program.methods
          .testHasOneWrapper()
          .accounts({
            data: dataKeypair.publicKey,
            authority: wrongAuthority.publicKey,
          })
          .signers([wrongAuthority])
          .rpc();
        assert.fail("Expected error for wrong authority");
      } catch (err) {
        assert.include(err.toString(), "ConstraintHasOne");
      }
    });
  });

});
