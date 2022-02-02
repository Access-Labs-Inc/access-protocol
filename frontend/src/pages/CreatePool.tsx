import React, { useState } from "react";
import Card from "../components/Card";
import { styled } from "@mui/material/styles";
import { createStakePool, ACCESS_PROGRAM_ID, StakePool } from "@access";
import { useWallet, useConnection } from "@solana/wallet-adapter-react";
import CircularProgress from "@mui/material/CircularProgress";
import FormControl from "@mui/material/FormControl";
import InputLabel from "@mui/material/InputLabel";
import OutlinedInput from "@mui/material/OutlinedInput";
import { Button } from "@mui/material";
import { PublicKey } from "@solana/web3.js";
import { sendTx } from "../utils/send";

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
  padding: 10,
  width: 500,
  height: 350,
  display: "flex",
  justifyContent: "space-around",
  alignItems: "center",
  flexDirection: "column",
});

const Link = styled("a")({
  textDecoration: "underline",
  fontWeight: "bold",
  cursor: "pointer",
});

const FormControlStyled = styled(FormControl)({
  width: "90%",
});

const CreatePool = () => {
  const { connection } = useConnection();
  const { publicKey, sendTransaction } = useWallet();
  const [loading, setLoading] = useState(false);
  const [minimum, setMinimum] = useState<null | number>(null);
  const [destination, setDestination] = useState<string | null>(null);
  const [stakePool, setStakePool] = useState<string | null>(null);

  const handle = async () => {
    if (!publicKey || !minimum || !destination) return;
    try {
      setLoading(true);
      const ix = await createStakePool(
        connection,
        publicKey,
        new PublicKey(destination),
        minimum,
        publicKey,
        ACCESS_PROGRAM_ID
      );

      const tx = await sendTx(connection, publicKey, ix, sendTransaction);
      console.log(tx);
      const [key] = await StakePool.getKey(ACCESS_PROGRAM_ID, publicKey);
      setStakePool(key.toBase58());
    } catch (err) {
      console.log(err);
    } finally {
      setLoading(false);
    }
  };

  return (
    <Container>
      <CardContainer>
        <Card>
          <InnerCard>
            <FormControlStyled>
              <InputLabel>Rewards destination</InputLabel>
              <OutlinedInput
                type="text"
                id="component-outlined"
                value={destination}
                onChange={(e) => setDestination(e.target.value.trim())}
                label="Rewards destination"
              />
            </FormControlStyled>
            <FormControlStyled>
              <InputLabel>Minimum stake amount</InputLabel>
              <OutlinedInput
                type="number"
                id="component-outlined"
                value={minimum}
                onChange={(e) => setMinimum(parseInt(e.target.value) || 0)}
                label="Minimum stake amount"
              />
            </FormControlStyled>
            <Button variant="contained" onClick={handle}>
              {loading ? <CircularProgress color="inherit" /> : "Create"}
            </Button>
            {stakePool && (
              <>
                <span>Stake pool address:</span>
                <Link href={`/stake/${stakePool}`}>{stakePool}</Link>
              </>
            )}
          </InnerCard>
        </Card>
      </CardContainer>
    </Container>
  );
};

export default CreatePool;
