import * as anchor from "@anchor-lang/core";
import { Program } from "@anchor-lang/core";
import { IdlCommandsOne } from "../target/types/idl_commands_one";
import { IdlCommandsTwo } from "../target/types/idl_commands_two";
import { assert } from "chai";
import { execSync } from "child_process";
import * as fs from "fs";

describe("Test CLI IDL commands", () => {
  // Configure the client to use the local cluster.
  const provider = anchor.AnchorProvider.env();

  anchor.setProvider(provider);

  const programOne = anchor.workspace.IdlCommandsOne as Program<IdlCommandsOne>;
  const programTwo = anchor.workspace.IdlCommandsTwo as Program<IdlCommandsTwo>;

  // program-metadata args for init/upgrade (anchor idl init/upgrade skip localnet)
  const keypair = "keypairs/deployer-keypair.json";
  const rpc = "http://localhost:8899";

  it("Can fetch an IDL using the TypeScript client", async () => {
    const idl = await anchor.Program.fetchIdl(programOne.programId, provider);
    assert.deepEqual(idl, programOne.rawIdl);
  });

  it("Can fetch an IDL via the CLI", async () => {
    const idl = execSync(`anchor idl fetch ${programOne.programId}`).toString();
    assert.deepEqual(JSON.parse(idl), programOne.rawIdl);
  });

  it("Can write a new IDL using the upgrade command", async () => {
    // Note: Since anchor idl init/upgrade skip localnet, we need to deploy the IDL via program-metadata.
    // Upgrade the IDL of program one to the IDL of program two
    execSync(
      `program-metadata --keypair ${keypair} --rpc ${rpc} write idl ${programOne.programId} target/idl/idl_commands_two.json`,
      { stdio: "inherit" },
    );
    const idl = await anchor.Program.fetchIdl(programOne.programId, provider);
    assert.deepEqual(idl, programTwo.rawIdl);
  });

  it("Can close IDL account", async () => {
    execSync(`anchor idl close ${programOne.programId}`, { stdio: "inherit" });
    const idl = await anchor.Program.fetchIdl(programOne.programId, provider);
    assert.isNull(idl);
  });

  it("Can initialize super massive IDL account", async () => {
    // Note: Since anchor idl init/upgrade skip localnet, we need to deploy the IDL via program-metadata.
    execSync(
      `program-metadata --keypair ${keypair} --rpc ${rpc} write idl ${programOne.programId} testLargeIdl.json`,
      { stdio: "inherit" },
    );
    const idlActual = await anchor.Program.fetchIdl(
      programOne.programId,
      provider,
    );
    const idlExpected = JSON.parse(
      fs.readFileSync("testLargeIdl.json", "utf8"),
    );
    assert.deepEqual(idlActual, idlExpected);
  });
});
