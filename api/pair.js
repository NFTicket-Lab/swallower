import {Keyring} from '@polkadot/keyring';

const BOB = "5FHneW46xGXgs5mUiveU4sbTyGBzmstUspZC92UhjJM694ty";
const keyring = new Keyring({ type: 'sr25519' });
const pair = keyring.addFromUri('//Bob');
export default pair;