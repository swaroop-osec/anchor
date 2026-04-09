import anchorPkg from "@coral-xyz/anchor";
const { Program, BN, AnchorProvider } = anchorPkg;
import { assert } from "chai";
import * as fs from "fs";
import * as path from "path";
import { fileURLToPath } from "url";

describe("validate-trait", () => {
  const provider = AnchorProvider.env();
  anchorPkg.setProvider(provider);

  const __filename = fileURLToPath(import.meta.url);
  const __dirname = path.dirname(__filename);

  const idlPath = path.join(__dirname, "../target/idl/validate_trait.json");
  const idl = JSON.parse(fs.readFileSync(idlPath, "utf8"));
  const program = new Program(idl, provider);

  const authority = provider.wallet;

  it("Is initialized!", async () => {
    const myAccount = anchorPkg.web3.Keypair.generate();
    const amount = new BN(42);
    await program.methods
      .setData(amount)
      .accounts({
        myAccount: myAccount.publicKey,
        authority: authority.publicKey,
      })
      .signers([myAccount])
      .rpc();

    const account = await program.account.myAccount.fetch(myAccount.publicKey);
    assert.ok(account.data.eq(amount));
    assert.ok(account.authority.equals(authority.publicKey));
  });

  it("Fails automatic constraint (amount > 10)", async () => {
    const amount = new BN(5);
    const newAccount = anchorPkg.web3.Keypair.generate();
    try {
      await program.methods
        .setData(amount)
        .accounts({
          myAccount: newAccount.publicKey,
          authority: authority.publicKey,
        })
        .signers([newAccount])
        .rpc();
      assert.fail("Should have failed");
    } catch (err) {
      assert.include(err.message, "A raw constraint was violated");
    }
  });

  it("Fails raw constraint on non-init field (amount >= 1000)", async () => {
    const amount = new BN(1000);
    const newAccount = anchorPkg.web3.Keypair.generate();
    try {
      await program.methods
        .setData(amount)
        .accounts({
          myAccount: newAccount.publicKey,
          authority: authority.publicKey,
        })
        .signers([newAccount])
        .rpc();
      assert.fail("Should have failed");
    } catch (err) {
      assert.include(err.message, "A raw constraint was violated");
    }
  });

  it("Fails manual validation (amount == 42)", async () => {
    const myAccount = anchorPkg.web3.Keypair.generate();
    // First initialize
    await program.methods
      .setData(new BN(20))
      .accounts({
        myAccount: myAccount.publicKey,
        authority: authority.publicKey,
      })
      .signers([myAccount])
      .rpc();

    const amount = new BN(42);
    try {
      await program.methods
        .manualSetData(amount)
        .accounts({
          myAccount: myAccount.publicKey,
          authority: authority.publicKey,
        })
        .rpc();
      assert.fail("Should have failed");
    } catch (err) {
      // Reusing InstructionDidNotDeserialize error code (3003) for testing
      assert.include(
        err.message,
        "The program could not deserialize the given instruction",
      );
    }
  });

  it("Passes manual validation", async () => {
    const myAccount = anchorPkg.web3.Keypair.generate();
    // First initialize
    await program.methods
      .setData(new BN(20))
      .accounts({
        myAccount: myAccount.publicKey,
        authority: authority.publicKey,
      })
      .signers([myAccount])
      .rpc();

    const amount = new BN(100);
    await program.methods
      .manualSetData(amount)
      .accounts({
        myAccount: myAccount.publicKey,
        authority: authority.publicKey,
      })
      .rpc();

    const account = await program.account.myAccount.fetch(myAccount.publicKey);
    assert.ok(account.data.eq(amount));
  });

  it("Fails when account is not writable", async () => {
    const myAccount = anchorPkg.web3.Keypair.generate();
    await program.methods
      .setData(new BN(20))
      .accounts({
        myAccount: myAccount.publicKey,
        authority: authority.publicKey,
      })
      .signers([myAccount])
      .rpc();

    const ix = await program.methods
      .manualSetData(new BN(100))
      .accounts({
        myAccount: myAccount.publicKey,
        authority: authority.publicKey,
      })
      .instruction();

    // Strip the writable flag from my_account
    ix.keys = ix.keys.map((k) =>
      k.pubkey.equals(myAccount.publicKey) ? { ...k, isWritable: false } : k
    );

    const tx = new anchorPkg.web3.Transaction().add(ix);
    try {
      await anchorPkg.web3.sendAndConfirmTransaction(
        provider.connection,
        tx,
        [provider.wallet.payer, myAccount] // Add myAccount as signer
      );
      assert.fail("should have failed");
    } catch (err) {
      const errMsg = err.message || "";
      assert.ok(
        errMsg.includes("ConstraintMut") || 
        errMsg.includes("instruction modified data of a read-only account"),
        `Unexpected error message: ${errMsg}`
      );
    }
  });

  it("Fails manual validation with custom struct (amount == 666)", async () => {
    const myAccount = anchorPkg.web3.Keypair.generate();
    // First initialize
    await program.methods
      .setData(new BN(20))
      .accounts({
        myAccount: myAccount.publicKey,
        authority: authority.publicKey,
      })
      .signers([myAccount])
      .rpc();

    const amount = new BN(666);
    try {
      await program.methods
        .customStructSetData({ amount })
        .accounts({
          myAccount: myAccount.publicKey,
          authority: authority.publicKey,
        })
        .rpc();
      assert.fail("Should have failed");
    } catch (err) {
      assert.include(err.message, "InstructionDidNotDeserialize");
    }
  });

  it("Passes manual validation with custom struct", async () => {
    const myAccount = anchorPkg.web3.Keypair.generate();
    // First initialize
    await program.methods
      .setData(new BN(20))
      .accounts({
        myAccount: myAccount.publicKey,
        authority: authority.publicKey,
      })
      .signers([myAccount])
      .rpc();

    const amount = new BN(777);
    await program.methods
      .customStructSetData({ amount })
      .accounts({
        myAccount: myAccount.publicKey,
        authority: authority.publicKey,
      })
      .rpc();

    const account = await program.account.myAccount.fetch(myAccount.publicKey);
    assert.ok(account.data.eq(amount));
  });

  it("Fails raw constraint (my_account.data > amount)", async () => {
    const myAccount = anchorPkg.web3.Keypair.generate();
    // First initialize with data = 42
    await program.methods
      .setData(new BN(42))
      .accounts({
        myAccount: myAccount.publicKey,
        authority: authority.publicKey,
      })
      .signers([myAccount])
      .rpc();

    // Try to set balance with amount = 50 (fails 42 > 50)
    try {
      await program.methods
        .setBalance(new BN(50))
        .accounts({
          myAccount: myAccount.publicKey,
        })
        .rpc();
      assert.fail("Should have failed");
    } catch (err) {
      assert.include(err.message, "A raw constraint was violated");
    }
  });

  it("Passes raw constraint (my_account.data > amount)", async () => {
    const myAccount = anchorPkg.web3.Keypair.generate();
    // First initialize with data = 100
    await program.methods
      .setData(new BN(100))
      .accounts({
        myAccount: myAccount.publicKey,
        authority: authority.publicKey,
      })
      .signers([myAccount])
      .rpc();

    // Try to set balance with amount = 50 (passes 100 > 50)
    const newAmount = new BN(50);
    await program.methods
      .setBalance(newAmount)
      .accounts({
        myAccount: myAccount.publicKey,
      })
      .rpc();
    
    const account = await program.account.myAccount.fetch(myAccount.publicKey);
    assert.ok(account.data.eq(newAmount));
  });
});
