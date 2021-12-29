import {ApiPromise,WsProvider} from '@polkadot/api';
import {Keyring} from '@polkadot/keyring';


const connectSubstrate = async () => {
    const wsProvider = new WsProvider('ws://127.0.0.1:9944');
    const api = await ApiPromise.create({ provider: wsProvider, types: {} });
    return api;
  };

  const keyring = new Keyring({type:'sr25519'});
const bobpair = ()=>{
  return keyring.createFromUri('//Bob');
};
 const maindata = async () => {
    const api = await connectSubstrate();
    // 取得链上 meta-data. 去掉下面 comment 去看链上 meta-data. 是一个挺大的 JSON 文件
    const metadata = await api.rpc.state.getMetadata();
    //console.log(`Chain Metadata: ${JSON.stringify(metadata, null, 2)}`);
    const condata = `Chain Metadata: ${JSON.stringify(metadata, null, 2)}`;
    return condata;
  };

  export default connectSubstrate;

//   main()
//   .then(() => {
//     console.log("successfully exited");
//     process.exit(0);
//   })
//   .catch(err => {
//     console.log('error occur:', err);
//     process.exit(1);
//   })