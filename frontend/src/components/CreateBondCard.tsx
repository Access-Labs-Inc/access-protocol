import React, { useState } from "react";
import { createBond, ACCESS_PROGRAM_ID } from "@access";
import FormControl from "@mui/material/FormControl";
import InputLabel from "@mui/material/InputLabel";
import OutlinedInput from "@mui/material/OutlinedInput";
import Card from "./Card";
import { styled } from "@mui/material/styles";
import { useWallet, useConnection } from "@solana/wallet-adapter-react";
import { PublicKey } from "@solana/web3.js";
import { sendTx } from "../utils/send";
import { Button } from "@mui/material";
import { notify } from "../utils/notifications";
import CircularProgress from "@mui/material/CircularProgress";

const Title = styled("span")({
  fontSize: 25,
  fontWeight: "bold",
});

const CardContainer = styled("div")({
  height: 1050,
});

const FormControlStyled = styled(FormControl)({
  width: "90%",
  margin: 10,
});

const BondCard = () => {
  const [loading, setLoading] = useState(false);
  const [buyer, setBuyer] = useState<string | null>(null);
  const [amountSold, setAmountSold] = useState<number | null>(null);
  const [quoteAmount, setQuoteAmount] = useState<number | null>(null);
  const [quoteMint, setQuoteMint] = useState<string | null>(null);
  const [sellerTokenAcc, setSellerTokenAcc] = useState<string | null>(null);
  const [unlockStart, setUnlockStart] = useState<number | null>(null);
  const [unlockPeriod, setUnlockPeriod] = useState<number | null>(null);
  const [unlockAmount, setUnlockAmount] = useState<number | null>(null);
  const [lastUnlock, setLaskUnlockTime] = useState<number | null>(null);
  const [stakePool, setStakePool] = useState<string | null>(null);
  const [sellerIndex, setSellerIndex] = useState<number | null>(null);

  const { connection } = useConnection();
  const { publicKey, connected, sendTransaction } = useWallet();
  const handle = async () => {
    try {
      setLoading(true);
      if (
        !publicKey ||
        !buyer ||
        !amountSold ||
        !quoteAmount ||
        !quoteMint ||
        !sellerTokenAcc ||
        !unlockStart ||
        !unlockPeriod ||
        !unlockAmount ||
        !lastUnlock ||
        !stakePool ||
        sellerIndex === null
      ) {
        return notify({ message: "Missing input" });
      }
      const ix = await createBond(
        publicKey,
        new PublicKey(buyer),
        amountSold,
        quoteAmount,
        new PublicKey(quoteMint),
        new PublicKey(sellerTokenAcc),
        unlockStart,
        unlockPeriod,
        unlockAmount,
        new PublicKey(stakePool),
        sellerIndex,
        ACCESS_PROGRAM_ID
      );
      const tx = await sendTx(connection, publicKey, [ix], sendTransaction);
      console.log(tx);
      notify({ message: "Bond created", variant: "success" });
    } catch (err) {
      console.log(err);
      // @ts-ignore
      notify({ message: `Error ${err.message}` });
    } finally {
      setLoading(false);
    }
  };
  return (
    <CardContainer>
      <Card>
        <Title>Create bond</Title>
        <FormControlStyled>
          <InputLabel>Buyer address</InputLabel>
          <OutlinedInput
            type="text"
            id="component-outlined"
            value={buyer}
            onChange={(e) => setBuyer(e.target.value.trim())}
            label="Buyer address"
          />
        </FormControlStyled>
        <FormControlStyled>
          <InputLabel>ACCESS Tokens amount</InputLabel>
          <OutlinedInput
            type="number"
            id="component-outlined"
            value={amountSold}
            onChange={(e) => setAmountSold(parseInt(e.target.value))}
            label="ACCESS tokens amount"
          />
        </FormControlStyled>
        <FormControlStyled>
          <InputLabel>Bond Price (in quote tokens)</InputLabel>
          <OutlinedInput
            type="number"
            id="component-outlined"
            value={quoteAmount}
            onChange={(e) => setQuoteAmount(parseInt(e.target.value))}
            label="ACCESS tokens amount"
          />
        </FormControlStyled>
        <FormControlStyled>
          <InputLabel>Quote mint</InputLabel>
          <OutlinedInput
            type="text"
            id="component-outlined"
            value={quoteMint}
            onChange={(e) => setQuoteMint(e.target.value)}
            label="Quote mint"
          />
        </FormControlStyled>
        <FormControlStyled>
          <InputLabel>Seller token account</InputLabel>
          <OutlinedInput
            type="text"
            id="component-outlined"
            value={sellerTokenAcc}
            onChange={(e) => setSellerTokenAcc(e.target.value)}
            label="Seller token account"
          />
        </FormControlStyled>
        <FormControlStyled>
          <InputLabel>Unlock start date (unix timestamp in s)</InputLabel>
          <OutlinedInput
            type="number"
            id="component-outlined"
            value={unlockStart}
            onChange={(e) => setUnlockStart(parseInt(e.target.value))}
            label="Unlock start date (s)"
          />
        </FormControlStyled>
        <FormControlStyled>
          <InputLabel>Unlock period (in s)</InputLabel>
          <OutlinedInput
            type="number"
            id="component-outlined"
            value={unlockPeriod}
            onChange={(e) => setUnlockPeriod(parseInt(e.target.value))}
            label="Unlock period (s)"
          />
        </FormControlStyled>
        <FormControlStyled>
          <InputLabel>Unlock amount</InputLabel>
          <OutlinedInput
            type="number"
            id="component-outlined"
            value={unlockAmount}
            onChange={(e) => setUnlockAmount(parseInt(e.target.value))}
            label="Unlock amount"
          />
        </FormControlStyled>
        <FormControlStyled>
          <InputLabel>Last unlock time (unix timestamp in s)</InputLabel>
          <OutlinedInput
            type="number"
            id="component-outlined"
            value={lastUnlock}
            onChange={(e) => setLaskUnlockTime(parseInt(e.target.value))}
            label="Last unlock time"
          />
        </FormControlStyled>
        <FormControlStyled>
          <InputLabel>Stake pool</InputLabel>
          <OutlinedInput
            type="text"
            id="component-outlined"
            value={stakePool}
            onChange={(e) => setStakePool(e.target.value.trim())}
            label="Stake pool"
          />
        </FormControlStyled>
        <FormControlStyled>
          <InputLabel>Seller index</InputLabel>
          <OutlinedInput
            type="number"
            id="component-outlined"
            value={sellerIndex}
            onChange={(e) => setSellerIndex(parseInt(e.target.value))}
            label="Stake pool"
          />
        </FormControlStyled>
        <Button disabled={!connected} variant="contained" onClick={handle}>
          {loading ? <CircularProgress color="inherit" /> : "Create bond"}
        </Button>
      </Card>
    </CardContainer>
  );
};

export default BondCard;
