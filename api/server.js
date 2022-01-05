import express from 'express'
import dotenv from 'dotenv'
import { connectSubstrate, getAccount } from './index.js'
import { cryptoWaitReady, mnemonicGenerate } from '@polkadot/util-crypto';
import { Keyring } from '@polkadot/keyring';
// import bobpair from './pair.js';

dotenv.config({
    path: './config/config.env',
})

const app = express()

const PORT = process.env.PORT || 3000;
const api = await connectSubstrate();



const BOB_ADDRESS = "5FHneW46xGXgs5mUiveU4sbTyGBzmstUspZC92UhjJM694ty";


app.get("/api/metaData", async (req, res) => {
    // const api = await connectSubstrate();
    const [magicNumber, metadata] = await api.rpc.state.getMetadata();
    // const addr='5DTestUPts3kjeXSTMyerHihn1uwMfLj8vU8sqF7qYrFabHE';
    // const acct = await api.query.system.account(addr)
    res.status(200).json({ success: true, msg: metadata })
})

app.get("/swallower/admin", async (req, res) => {
    const admin = await api.query.swallower.admin();
    res.status(200).json({ admin: admin })
});

app.get("/swallower/assetAmount", async (req, res) => {
    const assetAmount = await api.query.swallower.assetAmount();
    res.status(200).json({ assetAmount: assetAmount })
});

app.get("/swallower/assetId", async (req, res) => {
    const assetId = await api.query.swallower.assetId();
    res.status(200).json({ assetId: assetId })
});

app.get("/swallower/battleZoneRewardMap", async (req, res) => {
    const battleZoneRewardMap = await api.query.swallower.battleZoneRewardMap(req.query.hash);
    res.status(200).json({ battleZoneRewardMap: battleZoneRewardMap })
});

app.get("/swallower/geneAmount", async (req, res) => {
    const geneAmount = await api.query.swallower.geneAmount();
    res.status(200).json({ geneAmount: geneAmount })
});

app.get("/swallower/manager", async (req, res) => {
    const manager = await api.query.swallower.manager();
    res.status(200).json({ manager: manager })
});

app.get("/swallower/ownerSwallower", async (req, res) => {
    const ownerSwallower = await api.query.swallower.ownerSwallower(req.query.accountId);
    res.status(200).json({ ownerSwallower: ownerSwallower })
});

app.get("/swallower/safeZone", async (req, res) => {
    const safeZone = await api.query.swallower.safeZone(req.query.hash);
    res.status(200).json({ safeZone: safeZone })
});

app.get("/swallower/swallowerAmount", async (req, res) => {
    const swallowerAmount = await api.query.swallower.swallowerAmount();
    res.status(200).json({ swallowerAmount: swallowerAmount })
});

app.get("/swallower/swallowerNo", async (req, res) => {
    const swallowerNo = await api.query.swallower.swallowerNo();
    res.status(200).json({ swallowerNo: swallowerNo })
});


app.get("/swallower/swallowers", async (req, res) => {
    const swallowers = await api.query.swallower.swallowers(req.query.hash);
    res.status(200).json({ swallowers: swallowers })
});

const handleEvents = ({ events = [], status }) =>{
    console.log('Transaction status:', status.type);

    if (status.isInBlock) {
        console.log('Included at block hash', status.asInBlock.toHex());
        console.log('Events:');

        events.forEach(({ event: { data, method, section }, phase }) => {
            let event_msg =  '\t'+ phase.toString()+ `: ${section}.${method}`+ data.toString();
            console.log(event_msg);
            result.push({ data, method, section });
        });
    } else if (status.isFinalized) {
        result.push(status.asFinalized.toHex());
        console.log('Finalized block hash', status.asFinalized.toHex());
        res.status(200).json({ mintSwallower: result });
    }
}

app.get("/swallower/mintSwallower", async (req, res) => {
    await cryptoWaitReady();
    var account = req.query.account;
    var account = getAccount(account);
    console.log("req.query.account is:", account);
    // Get the nonce for the admin key
    const { nonce } = await api.query.system.account(account.address);
    const mintSwallower = await api.tx.swallower.mintSwallower(req.query.name);
    let result = new Array();
    const handleEvents = ({ events = [], status }) => {
        console.log('Transaction status:', status.type);

        if (status.isInBlock) {
            console.log('Included at block hash', status.asInBlock.toHex());
            console.log('Events:');

            events.forEach(({ event: { data, method, section }, phase }) => {
                let event_msg =  '\t'+ phase.toString()+ `: ${section}.${method}`+ data.toString();
                console.log(event_msg);
                result.push({ data, method, section });
            });
        } else if (status.isFinalized) {
            result.push(status.asFinalized.toHex());
            console.log('Finalized block hash', status.asFinalized.toHex());
            res.status(200).json({ mintSwallower: result });
        }
        
    }

    await mintSwallower.signAndSend(account,handleEvents );

});

app.get("/swallower/changeSwallowerName", async (req, res) => {
    await cryptoWaitReady();
    var account = req.query.account;
    var account = getAccount(account);
    let hash = req.query.hash;
    if (!hash){
        res.status(200).json({ error: "hash is required!" });
        return;
    }
    let name = req.query.name;
    // Get the nonce for the admin key
    const changeSwallowerName = await api.tx.swallower.changeSwallowerName(hash,name);
    let result = new Array();
    const handleEvents = ({ events = [], status }) => {
        console.log('Transaction status:', status.type);

        if (status.isInBlock) {
            console.log('Included at block hash', status.asInBlock.toHex());
            console.log('Events:');

            events.forEach(({ event: { data, method, section }, phase }) => {
                let event_msg =  '\t'+ phase.toString()+ `: ${section}.${method}`+ data.toString();
                console.log(event_msg);
                result.push({ data, method, section });
            });
        } else if (status.isFinalized) {
            result.push(status.asFinalized.toHex());
            console.log('Finalized block hash', status.asFinalized.toHex());
            res.status(200).json({ result: result });
        }
        
    }

    await changeSwallowerName.signAndSend(account,handleEvents );

});

app.get("/swallower/burnSwallower", async (req, res) => {
    await cryptoWaitReady();
    var account = req.query.account;
    var account = getAccount(account);
    let hash = req.query.hash;
    if (!hash){
        res.status(200).json({ error: "hash is required!" });
        return;
    }
    let name = req.query.name;
    // Get the nonce for the admin key
    const burnSwallower = await api.tx.swallower.burnSwallower(hash);
    let result = new Array();
    const handleEvents = ({ events = [], status }) => {
        console.log('Transaction status:', status.type);

        if (status.isInBlock) {
            console.log('Included at block hash', status.asInBlock.toHex());
            console.log('Events:');

            events.forEach(({ event: { data, method, section }, phase }) => {
                let event_msg =  '\t'+ phase.toString()+ `: ${section}.${method}`+ data.toString();
                console.log(event_msg);
                result.push({ data, method, section });
            });
        } else if (status.isFinalized) {
            result.push(status.asFinalized.toHex());
            console.log('Finalized block hash', status.asFinalized.toHex());
            res.status(200).json({ result: result });
        }
        
    }

    await burnSwallower.signAndSend(account,handleEvents );

});



app.listen(PORT, console.log(`Server running in ${process.env.NODE_ENV} mode on port ${PORT}`))