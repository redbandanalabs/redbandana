import {
  Connection,
  Keypair,
  PublicKey,
  Transaction,
  TransactionInstruction,
  SystemProgram,
} from "@solana/web3.js";
import * as borsh from "borsh";

enum ProofSystem {
  Groth16 = 0,
  Stark = 1,
}

class InitializeInstruction {
  variant = 0;
}
class DepositInstruction {
  variant = 1;
  commitment: Uint8Array;

  constructor(comm: Uint8Array) {
    this.commitment = comm;
  }
}

const InitializeSchema = new Map([[InitializeInstruction, { kind: "struct", fields: [["variant", "u8"]] }]]);
const DepositSchema = new Map([
  [
    DepositInstruction,
    {
      kind: "struct",
      fields: [
        ["variant", "u8"],
        ["commitment", [32]],
      ],
    },
  ],
]);

async function main() {
  const connection = new Connection("https://api.devnet.solana.com", "confirmed");
  const payer = Keypair.generate();

  // In real use, fund payer via airdrop or faucet.
  const airdropSig = await connection.requestAirdrop(payer.publicKey, 1e9);
  await connection.confirmTransaction(airdropSig, "confirmed");

  const programId = new PublicKey("<DEPLOYED_PROGRAM_ID>");

  const [globalStatePda] = PublicKey.findProgramAddressSync(
    [Buffer.from("global-state")],
    programId
  );

  // Initialize
  {
    const ixData = Buffer.from(borsh.serialize(InitializeSchema, new InitializeInstruction()));
    const ix = new TransactionInstruction({
      programId,
      keys: [
        { pubkey: globalStatePda, isSigner: false, isWritable: true },
        { pubkey: payer.publicKey, isSigner: true, isWritable: true },
        { pubkey: SystemProgram.programId, isSigner: false, isWritable: false },
      ],
      data: ixData,
    });
    const tx = new Transaction().add(ix);
    await connection.sendTransaction(tx, [payer]);
  }

  // Deposit
  {
    const commitment = new Uint8Array(32).fill(1);
    const [notePda] = PublicKey.findProgramAddressSync(
      [Buffer.from("note"), Buffer.from(commitment)],
      programId
    );
    const ixData = Buffer.from(borsh.serialize(DepositSchema, new DepositInstruction(commitment)));
    const ix = new TransactionInstruction({
      programId,
      keys: [
        { pubkey: globalStatePda, isSigner: false, isWritable: true },
        { pubkey: payer.publicKey, isSigner: true, isWritable: true },
        { pubkey: notePda, isSigner: false, isWritable: true },
        { pubkey: SystemProgram.programId, isSigner: false, isWritable: false },
      ],
      data: ixData,
    });
    const tx = new Transaction().add(ix);
    await connection.sendTransaction(tx, [payer]);
  }

  console.log("Initialized + deposited commitment");
}

main().catch(console.error);
