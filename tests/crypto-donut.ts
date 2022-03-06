import * as anchor from '@project-serum/anchor';
import {Program} from '@project-serum/anchor';
import {CryptoDonut} from '../target/types/crypto_donut';
import assert from "assert";

const {PublicKey} = anchor.web3;

describe('crypto-donut', () => {
    const provider = anchor.Provider.env();
    anchor.setProvider(provider);

    const program = anchor.workspace.CryptoDonut as Program<CryptoDonut>;
    const authority = program.provider.wallet.publicKey;
    const contributor = anchor.web3.Keypair.generate();

    it('create wallet!', async () => {
        const [wallet, _walletBump] = await PublicKey.findProgramAddress(
            [authority.toBuffer()],
            program.programId
        );

        await program.rpc.createWallet({
            accounts: {
                wallet: wallet,
                authority: authority,
                systemProgram: anchor.web3.SystemProgram.programId,
            },
        });

        const account = await program.account.wallet.fetch(wallet);
        assert.ok(account.authority.equals(authority));
    });

    it('create ledger!', async () => {
        const [wallet, _walletBump] = await PublicKey.findProgramAddress(
            [authority.toBuffer()],
            program.programId
        );

        const [ledger, _ledgerBump] = await PublicKey.findProgramAddress(
            [wallet.toBuffer()],
            program.programId
        );

        await program.rpc.createLedger({
            accounts: {
                wallet,
                ledger,
                authority,
                systemProgram: anchor.web3.SystemProgram.programId,
            }
        });

    });

    it('make donate!', async () => {
        const [wallet, _walletBump] = await PublicKey.findProgramAddress(
            [authority.toBuffer()],
            program.programId
        );

        const [ledger, _ledgerBump] = await PublicKey.findProgramAddress(
            [wallet.toBuffer()],
            program.programId
        );

        const to = wallet;
        const from = contributor
        const amount = new anchor.BN(1000000);

        await provider.connection.confirmTransaction(
            await provider.connection.requestAirdrop(from.publicKey, 1000000000)
        );

        let beforeBalance = (
            await program.provider.connection.getAccountInfo(
                to
            )
        ).lamports;

        await program.rpc.donate(amount, {
            accounts: {
                from: from.publicKey,
                to,
                ledger,
                systemProgram: anchor.web3.SystemProgram.programId,
            },
            signers: [from]
        });

        let afterBalance = (
            await program.provider.connection.getAccountInfo(
                to
            )
        ).lamports;

        assert.ok(beforeBalance != afterBalance);
    });

    it('withdraw!', async () => {
        const [wallet, _walletBump] = await PublicKey.findProgramAddress(
            [authority.toBuffer()],
            program.programId
        );

        let beforeBalance = (
            await program.provider.connection.getAccountInfo(
                authority
            )
        ).lamports;

        await program.rpc.withdraw({
            accounts: {
                wallet,
                authority,
            },
        });

        let afterBalance = (
            await program.provider.connection.getAccountInfo(
                authority
            )
        ).lamports;

        assert.ok(beforeBalance != afterBalance);

        try {
            const thief = anchor.web3.Keypair.generate();

            await program.rpc.withdraw({
                accounts: {
                    wallet,
                    authority: thief.publicKey,
                },
                signers: [thief]
            });

            assert.ok(false);
        } catch (err) {
        }
    });

    it('list contributors!', async () => {
        const [wallet, _walletBump] = await PublicKey.findProgramAddress(
            [authority.toBuffer()],
            program.programId
        );

        const [ledger, _ledgerBump] = await PublicKey.findProgramAddress(
            [wallet.toBuffer()],
            program.programId
        );

        const ledgerData = await program.account.ledger.fetch(ledger);
        // ha-ha..
        console.log(ledgerData.contributors);
    });
});
