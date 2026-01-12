import * as anchor from "@anchor-lang/core";
import type { ConstraintsTrait } from "../target/types/constraints_trait";

describe("constraints-trait", () => {
  anchor.setProvider(anchor.AnchorProvider.env());
  const program = anchor.workspace
    .constraintsTrait as anchor.Program<ConstraintsTrait>;
  const provider = anchor.getProvider() as anchor.AnchorProvider;

  it("runs constraints validation", async () => {
    await program.methods
      .noop()
      .accounts({ authority: provider.wallet.publicKey })
      .rpc();
  });
});
