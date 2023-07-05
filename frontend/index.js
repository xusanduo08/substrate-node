const { ApiPromise, WsProvider } = require ('@polkadot/api');
// import { u8aToString } from '@polkadot/util';
const { Tuple, u8, u64, Vec, Struct } = require('@polkadot/types-codec');


async function createApi() {
    const wsProvider = new WsProvider('ws://127.0.0.1:43575');
    const api = await ApiPromise.create({ provider: wsProvider, types: { IndexingData: { name: 'Vec<u8>', number: 'u64'  }} });
    const storage = await api.rpc.offchain.localStorageGet("PERSISTENT", "ocw-demo::storage::Tx");

    const struct = api.registry.createType('IndexingData', storage.toJSON());

    console.log("====== storage", struct.toHuman());
    process.exit(1);
}

createApi();