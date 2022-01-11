import test from 'tape';

import { DataV2, MetadataDataData, UpdateMetadataV2, VerifyCollection } from '../src/mpl-token-metadata';
import {
    killStuckProcess,
    initMetadata,
    getMetadataData,
    assertMetadataDataUnchanged,
    assertMetadataDataDataUnchanged,
    URI,
    NAME,
    SYMBOL,
    connectionURL,
    SELLER_FEE_BASIS_POINTS,
} from './utils';
import { airdrop, assertError, PayerTransactionHandler, TransactionHandler } from '@metaplex-foundation/amman';
import { Connection, Keypair, PublicKey } from '@solana/web3.js';
import { mintAndCreateMetadata, createMasterEdition } from './actions';
import { Collection } from '../src/accounts';

killStuckProcess();

async function createCollection(
    connection: Connection,
    transactionHandler: TransactionHandler,
    payer: Keypair,
) {
    const initMetadataData = new DataV2({
        uri: URI,
        name: NAME,
        symbol: SYMBOL,
        sellerFeeBasisPoints: SELLER_FEE_BASIS_POINTS,
        creators: null,
        collection: null,
        uses: null,
    });
    return await createMasterEdition(
        connection,
        transactionHandler,
        payer,
        initMetadataData,
    );
}


// -----------------
// Success Cases
// -----------------

test('verify-collection', async (t) => {
    const payer = Keypair.generate();
    const connection = new Connection(connectionURL, 'confirmed');
    const transactionHandler = new PayerTransactionHandler(connection, payer);

    await airdrop(connection, payer.publicKey, 2);

    let collectionNft = await createCollection(connection, transactionHandler, payer);

    const initMetadataData = new DataV2({
        uri: URI,
        name: NAME,
        symbol: SYMBOL,
        sellerFeeBasisPoints: SELLER_FEE_BASIS_POINTS,
        creators: null,
        collection: new Collection({ key: collectionNft.mint.publicKey.toBase58(), verified: false }),
        uses: null,
    });
    let collectionMemberNft = await createMasterEdition(
        connection,
        transactionHandler,
        payer,
        initMetadataData,
    );
    console.log('collectionMemberNft', collectionMemberNft.metadata.toBase58());
    const updatedMetadataBeforeVerification = await getMetadataData(connection, collectionMemberNft.metadata);
    t.ok(updatedMetadataBeforeVerification.collection, 'collection should be not null');
    t.not(updatedMetadataBeforeVerification.collection.verified, 'collection should be not be verified');
    const collectionVerifyCollectionTransaction = new VerifyCollection({ feePayer: payer.publicKey }, {
        metadata: collectionMemberNft.metadata,
        collectionAuthority: payer.publicKey,
        collectionMint: collectionNft.mint.publicKey,
        collectionMetadata: collectionNft.metadata,
        collectionMasterEdition: collectionNft.masterEditionPubkey,
    });
    const tx = await transactionHandler.sendAndConfirmTransaction(collectionVerifyCollectionTransaction, [payer], { skipPreflight: true });
    const updatedMetadataAfterVerification = await getMetadataData(connection, collectionMemberNft.metadata);
    t.ok(updatedMetadataAfterVerification.collection, 'collection should be not null');
    t.ok(updatedMetadataAfterVerification.collection.verified, 'collection should be verified');
});