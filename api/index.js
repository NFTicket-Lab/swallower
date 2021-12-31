import {ApiPromise,WsProvider} from '@polkadot/api';
import {Keyring} from '@polkadot/keyring';
import { cryptoWaitReady, mnemonicGenerate } from '@polkadot/util-crypto';

// we only need to do this once per app, somewhere in our init code
// (when using the API and waiting on `isReady` this is done automatically)
await cryptoWaitReady();

const connectSubstrate = async () => {
    const wsProvider = new WsProvider('ws://127.0.0.1:9944');
    const api = await ApiPromise.create({ provider: wsProvider, types: {} });
    return api;
  };

  // const keyring = new Keyring({type:'sr25519'});
const getAccount = (account)=>{
  return accountMap.get(account);
  // console.log("accountMap is:",accountMap);
};

var accountMap = new Map();
const keyring = new Keyring({ type: 'sr25519' });

const initAccount = ()=>{
  const ALICE = keyring.addFromUri('//Alice');
  const BOB = keyring.addFromUri('//Bob');
  const CHARLIE = keyring.addFromUri('//Charlie');
  const DAVE = keyring.addFromUri('//Dave');
  const EVE = keyring.addFromUri('//Eve');
  const FERDIE = keyring.addFromUri('//Ferdie');
  accountMap.set('ALICE',ALICE);
  accountMap.set('BOB',BOB);
  accountMap.set('CHARLIE',CHARLIE);
  accountMap.set('DAVE',DAVE);
  accountMap.set('EVE',EVE);
  accountMap.set('FERDIE',FERDIE);
}

initAccount();


 const maindata = async () => {
    const api = await connectSubstrate();
    // 取得链上 meta-data. 去掉下面 comment 去看链上 meta-data. 是一个挺大的 JSON 文件
    const metadata = await api.rpc.state.getMetadata();
    //console.log(`Chain Metadata: ${JSON.stringify(metadata, null, 2)}`);
    const condata = `Chain Metadata: ${JSON.stringify(metadata, null, 2)}`;
    return condata;
  };

  export {connectSubstrate,getAccount};

//   main()
//   .then(() => {
//     console.log("successfully exited");
//     process.exit(0);
//   })
//   .catch(err => {
//     console.log('error occur:', err);
//     process.exit(1);
//   })