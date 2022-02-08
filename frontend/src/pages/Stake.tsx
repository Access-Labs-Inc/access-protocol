import React, { useState } from "react";
import Card from "../components/Card";
import { styled } from "@mui/material/styles";
import { useWallet, useConnection } from "@solana/wallet-adapter-react";
import CircularProgress from "@mui/material/CircularProgress";
import FormControl from "@mui/material/FormControl";
import InputLabel from "@mui/material/InputLabel";
import OutlinedInput from "@mui/material/OutlinedInput";
import { Button } from "@mui/material";
import { PublicKey } from "@solana/web3.js";
import { sendTx } from "../utils/send";
import { createStakeAccount, ACCESS_PROGRAM_ID, StakeAccount } from "@access";
import { notify } from "../utils/notifications";

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

const Title = styled("span")({
  fontSize: 25,
});

const FormControlStyled = styled(FormControl)({
  width: "90%",
});

const Stake = () => {
  const { connection } = useConnection();
  const { publicKey, sendTransaction } = useWallet();
  const [stakePool, setStakePool] = useState<string | null>(null);
  const [loading, setLoading] = useState(false);
  const [stakeAcc, setStakeAcc] = useState<string | null>(null);

  const handle = async () => {
    if (!publicKey || !stakePool) return;
    try {
      setLoading(true);
      const ix = await createStakeAccount(
        new PublicKey(stakePool),
        publicKey,
        publicKey,
        ACCESS_PROGRAM_ID
      );
      const tx = await sendTx(connection, publicKey, [ix], sendTransaction);
      const [stakeKey] = await StakeAccount.getKey(
        ACCESS_PROGRAM_ID,
        publicKey,
        new PublicKey(stakePool)
      );
      console.log(tx);
      setStakeAcc(stakeKey.toBase58());
      notify({ message: `Tokens staked`, variant: "success" });
    } catch (err) {
      console.log(err);
      notify({ message: `Error ${err}`, variant: "error" });
    } finally {
      setLoading(false);
    }
  };

  return (
    <Container>
      <CardContainer>
        <Card>
          <InnerCard>
            <Title>Create stake account</Title>
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
            <Button variant="contained" onClick={handle}>
              {loading ? <CircularProgress color="inherit" /> : "Create"}
            </Button>
            {stakeAcc && (
              <>
                <span>Stake account address:</span>
                <strong>{stakeAcc}</strong>
              </>
            )}
          </InnerCard>
        </Card>
      </CardContainer>
    </Container>
  );
};

export default Stake;
