import React, { useState } from "react";
import FormControl from "@mui/material/FormControl";
import InputLabel from "@mui/material/InputLabel";
import OutlinedInput from "@mui/material/OutlinedInput";
import Card from "./Card";
import { styled } from "@mui/material/styles";
import { adminMint, ACCESS_PROGRAM_ID } from "@access";
import { useWallet, useConnection } from "@solana/wallet-adapter-react";
import { PublicKey } from "@solana/web3.js";
import { sendTx } from "../utils/send";
import { Button } from "@mui/material";
import { notify } from "../utils/notifications";
import CircularProgress from "@mui/material/CircularProgress";
import { refreshAllCaches } from "../utils/fetch-loop";

const Title = styled("span")({
  fontSize: 25,
  fontWeight: "bold",
});

const CardContainer = styled("div")({
  height: 350,
});

const FormControlStyled = styled(FormControl)({
  width: "90%",
  margin: 10,
});

const AdminMint = () => {
  const [loading, setLoading] = useState(false);
  const [destination, setDestination] = useState<string | null>(null);
  const [amount, setAmount] = useState<number | null>(null);
  const { connection } = useConnection();
  const { publicKey, sendTransaction, connected } = useWallet();

  const handleMint = async () => {
    if (!amount || !destination || !publicKey) return;
    try {
      setLoading(true);

      const ix = await adminMint(
        connection,
        amount * Math.pow(10, 6), // Assumes 6 decimals
        new PublicKey(destination),
        ACCESS_PROGRAM_ID
      );
      const tx = await sendTx(connection, publicKey, [ix], sendTransaction);
      console.log(tx);
      notify({
        message: "Tokens minted",
        variant: "success",
      });
    } catch (err) {
      console.log(err);
      // @ts-ignore
      notify({ message: `Error ${err.message}`, variant: "error" });
    } finally {
      setLoading(false);
      refreshAllCaches();
    }
  };

  return (
    <CardContainer>
      <Card>
        <Title>Admin mint</Title>
        <FormControlStyled>
          <InputLabel>Destination</InputLabel>
          <OutlinedInput
            type="text"
            id="component-outlined"
            value={destination}
            onChange={(e) => setDestination(e.target.value.trim())}
            label="Destination"
          />
        </FormControlStyled>
        <FormControlStyled>
          <InputLabel>Amount</InputLabel>
          <OutlinedInput
            type="number"
            id="component-outlined"
            value={amount}
            onChange={(e) => setAmount(parseInt(e.target.value))}
            label="Amount"
          />
        </FormControlStyled>
        <Button disabled={!connected} variant="contained" onClick={handleMint}>
          {loading ? <CircularProgress color="inherit" /> : "Mint"}
        </Button>
      </Card>
    </CardContainer>
  );
};

export default AdminMint;
