import React, { useState } from "react";
import FormControl from "@mui/material/FormControl";
import InputLabel from "@mui/material/InputLabel";
import OutlinedInput from "@mui/material/OutlinedInput";
import Card from "../components/Card";
import { styled } from "@mui/material/styles";
import { adminMint, ACCESS_PROGRAM_ID } from "@access";
import { useWallet, useConnection } from "@solana/wallet-adapter-react";
import { PublicKey } from "@solana/web3.js";
import { sendTx } from "../utils/send";
import { Button } from "@mui/material";

const Title = styled("span")({
  fontSize: 25,
  fontWeight: "bold",
});

const Container = styled("div")({
  display: "flex",
  alignItems: "center",
  justifyContent: "center",
  height: "100vh",
  flexDirection: "column",
});

const CardContainer = styled("div")({
  height: 300,
});

const InnerCard = styled("div")({
  padding: 20,
  width: 900,
  height: 450,
  display: "flex",
  justifyContent: "space-around",
  alignItems: "center",
  flexDirection: "column",
});

const FormControlStyled = styled(FormControl)({
  width: "90%",
  margin: 10,
});

const AdminMint = () => {
  const [destination, setDestination] = useState<string | null>(null);
  const [amount, setAmount] = useState<number | null>(null);
  const { connection } = useConnection();
  const { publicKey, sendTransaction } = useWallet();

  const handleMint = async () => {
    if (!amount || !destination || !publicKey) return;
    try {
      const ix = await adminMint(
        connection,
        amount,
        new PublicKey(destination),
        ACCESS_PROGRAM_ID
      );
      const tx = await sendTx(connection, publicKey, [ix], sendTransaction);
      console.log(tx);
    } catch (err) {
      console.log(err);
    }
  };

  return (
    <Container>
      <CardContainer>
        <Card>
          <InnerCard>
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
            <Button variant="contained" onClick={handleMint}>
              Mint
            </Button>
          </InnerCard>
        </Card>
      </CardContainer>
    </Container>
  );
};

export default AdminMint;
