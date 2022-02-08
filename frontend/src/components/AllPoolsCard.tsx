import React, { useState } from "react";
import { useAllActivePools, useAllInactivePools } from "../hooks/useAllPools";
import { styled } from "@mui/material/styles";
import { PublicKey } from "@solana/web3.js";
import { activateStakePool, ACCESS_PROGRAM_ID } from "@access";
import { useConnection, useWallet } from "@solana/wallet-adapter-react";
import { Button } from "@mui/material";
import { sendTx } from "../utils/send";
import CircularProgress from "@mui/material/CircularProgress";
import { useNavigate } from "react-router-dom";
import { notify } from "../utils/notifications";
import Card from "../components/Card";

const CardContainer = styled("div")({
  height: 400,
  display: "flex",
  alignItems: "center",
  justifyContent: "center",
  flexDirection: "column",
});

const Section = styled("span")({
  fontSize: 25,
  fontWeight: "bold",
});

const Row = styled("div")({
  border: "1px solid",
  borderColor: "black",
  borderRadius: 10,
  padding: 10,
  display: "flex",
  alignItems: "center",
});

const AllPools = () => {
  const [loading, setLoading] = useState(false);
  const { publicKey, sendTransaction } = useWallet();
  const { connection } = useConnection();
  const [pools] = useAllActivePools();
  const [inactivePools] = useAllInactivePools();
  const navigate = useNavigate();

  const handleActivate = (pool: PublicKey) => async () => {
    if (!publicKey) return;
    try {
      setLoading(true);
      const ix = await activateStakePool(connection, pool, ACCESS_PROGRAM_ID);
      const tx = await sendTx(connection, publicKey, [ix], sendTransaction);
      notify({ message: "Pool activated", variant: "success" });
    } catch (err) {
      console.log(err);
      notify({ message: `Error ${err}`, variant: "error" });
    } finally {
      setLoading(false);
    }
  };

  return (
    <CardContainer>
      <Card>
        <Section>Active stake pools</Section>
        {pools?.map((p) => {
          return (
            <Row onClick={() => navigate(`/pool/${p.pubkey.toBase58()}`)}>
              {p.pubkey.toBase58()}
            </Row>
          );
        })}
        <Section>Inactive stake pools</Section>
        {inactivePools?.map((p) => {
          return (
            <Row onClick={handleActivate(p.pubkey)}>
              {p.pubkey.toBase58()}
              <Button>{loading ? <CircularProgress /> : "Activate"}</Button>
            </Row>
          );
        })}
      </Card>
    </CardContainer>
  );
};

export default AllPools;
