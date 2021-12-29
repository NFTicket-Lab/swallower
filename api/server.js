import express from 'express'
import dotenv from 'dotenv'
import connectSubstrate from './index.js'

dotenv.config({
    path:'./config/config.env',
})

const app = express()

const PORT = process.env.PORT||3000;


app.get("/api",async (req,res)=>{
    const api = await connectSubstrate();
    //const {magicNumber,metadata} = await api.rpc.state.getMetadata();
    const addr='5DTestUPts3kjeXSTMyerHihn1uwMfLj8vU8sqF7qYrFabHE';
    const acct = await api.query.system.account(addr)
    res.status(200).json({success:true,msg:acct})
})

app.listen(PORT,console.log(`Server running in ${process.env.NODE_ENV} mode on port ${PORT}`))