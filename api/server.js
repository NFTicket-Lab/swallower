import express from 'express'
import dotenv from 'dotenv'
import connectSubstrate from './index.js'
import { cryptoWaitReady, mnemonicGenerate } from '@polkadot/util-crypto';
import {Keyring} from '@polkadot/keyring';
// import bobpair from './pair.js';

dotenv.config({
    path:'./config/config.env',
})

const app = express()

const PORT = process.env.PORT||3000;
const api = await connectSubstrate();

app.get("/api",async (req,res)=>{
    // const api = await connectSubstrate();
    const [magicNumber,metadata] = await api.rpc.state.getMetadata();
    // const addr='5DTestUPts3kjeXSTMyerHihn1uwMfLj8vU8sqF7qYrFabHE';
    // const acct = await api.query.system.account(addr)
    res.status(200).json({success:true,msg:metadata})
})

app.get("/swallower/admin",async (req,res)=>{
    const admin = await api.query.swallower.admin();
    res.status(200).json({admin:admin})
});

app.get("/swallower/assetAmount",async (req,res)=>{
    const assetAmount = await api.query.swallower.assetAmount();
    res.status(200).json({assetAmount:assetAmount})
});

app.get("/swallower/assetId",async (req,res)=>{
    const assetId = await api.query.swallower.assetId();
    res.status(200).json({assetId:assetId})
});

app.get("/swallower/battleZoneRewardMap",async (req,res)=>{
    const battleZoneRewardMap = await api.query.swallower.battleZoneRewardMap(req.query.hash);
    res.status(200).json({battleZoneRewardMap:battleZoneRewardMap})
});

app.get("/swallower/geneAmount",async (req,res)=>{
    const geneAmount = await api.query.swallower.geneAmount();
    res.status(200).json({geneAmount:geneAmount})
});

app.get("/swallower/manager",async (req,res)=>{
    const manager = await api.query.swallower.manager();
    res.status(200).json({manager:manager})
});

app.get("/swallower/ownerSwallower",async (req,res)=>{
    const ownerSwallower = await api.query.swallower.ownerSwallower(req.query.accountId);
    res.status(200).json({ownerSwallower:ownerSwallower})
});

app.get("/swallower/safeZone",async (req,res)=>{
    const safeZone = await api.query.swallower.safeZone(req.query.hash);
    res.status(200).json({safeZone:safeZone})
});

app.get("/swallower/swallowerAmount",async (req,res)=>{
    const swallowerAmount = await api.query.swallower.swallowerAmount();
    res.status(200).json({swallowerAmount:swallowerAmount})
});

app.get("/swallower/swallowerNo",async (req,res)=>{
    const swallowerNo = await api.query.swallower.swallowerNo();
    res.status(200).json({swallowerNo:swallowerNo})
});


app.get("/swallower/swallowers",async (req,res)=>{
    const swallowers = await api.query.swallower.swallowers(req.query.hash);
    res.status(200).json({swallowers:swallowers})
});

app.get("/swallower/mintSwallower",async (req,res)=>{
    //api.rx.swallower.mintSwallower()
    const value = 3000n * 1000000n;
    const gasLimit = 3000n * 1000000n;//不限制gas
    await cryptoWaitReady(); 
    const BOB_ADDRESS = "5FHneW46xGXgs5mUiveU4sbTyGBzmstUspZC92UhjJM694ty";
    const keyring = new Keyring({ type: 'sr25519' });
    const BOB = keyring.addFromUri('//Bob');
    // Get the nonce for the admin key
    const { nonce } = await api.query.system.account(BOB_ADDRESS);
    const mintSwallower = await api.tx.swallower.mintSwallower(req.query.name);
    mintSwallower.signAndSend(BOB, { nonce }, ({ events = [], status }) => {
        console.log('Transaction status:', status.type);
  
        if (status.isInBlock) {
          console.log('Included at block hash', status.asInBlock.toHex());
          console.log('Events:');
  
          events.forEach(({ event: { data, method, section }, phase }) => {
            console.log('\t', phase.toString(), `: ${section}.${method}`, data.toString());
          });
        } else if (status.isFinalized) {
          console.log('Finalized block hash', status.asFinalized.toHex());
  
        //   process.exit(0);
        }
      });

    res.status(200).json({mintSwallower:"SUCCESS"})
});

app.listen(PORT,console.log(`Server running in ${process.env.NODE_ENV} mode on port ${PORT}`))