import React, { useState } from "react";
import { useAllActiveBonds, useAllInactiveBonds } from "../hooks/useAllBonds";
import { styled } from "@mui/material/styles";
import Card from "./Card";
import { useConnection, useWallet } from "@solana/wallet-adapter-react";
import { Button } from "@mui/material";
import { sendTx } from "../utils/send";
import CircularProgress from "@mui/material/CircularProgress";
import { useNavigate } from "react-router-dom";
import { notify } from "../utils/notifications";
import { PublicKey } from "@solana/web3.js";
import { refreshAllCaches } from "../utils/fetch-loop";
import { signBond, ACCESS_PROGRAM_ID } from "@access";

const Section = styled("span")({
  fontSize: 25,
  fontWeight: "bold",
});

const CardContainer = styled("div")({
  height: 400,
});

const Row = styled("div")({
  border: "1px solid",
  borderColor: "black",
  borderRadius: 10,
  padding: 10,
  display: "flex",
  alignItems: "center",
});

const AllBonds = () => {
  const [loading, setLoading] = useState(false);
  const { publicKey, sendTransaction } = useWallet();
  const { connection } = useConnection();
  const [activeBonds] = useAllActiveBonds();
  const [inactiveBonds] = useAllInactiveBonds();
  const navigate = useNavigate();

  const handleSign = (bond: PublicKey) => async () => {
    if (!publicKey) return;
    try {
      setLoading(true);
      const ix = await signBond(0, publicKey, bond, ACCESS_PROGRAM_ID);
      const tx = await sendTx(connection, publicKey, [ix], sendTransaction);
      console.log(tx);
    } catch (err) {
      console.log(err);
      notify({ message: `Error ${err}`, variant: "error" });
    } finally {
      setLoading(false);
      refreshAllCaches();
    }
  };

  return (
    <CardContainer>
      <Card>
        <Section>Active bonds</Section>
        {activeBonds?.map((b) => {
          return (
            <Row onClick={() => navigate(`/bond/${b.key.toBase58()}`)}>
              {b.key.toBase58()}
            </Row>
          );
        })}
        <Section>Inactive bonds</Section>
        {inactiveBonds?.map((b) => {
          return (
            <Row onClick={() => navigate(`/bond/${b.key.toBase58()}`)}>
              {b.key.toBase58()}
              <Button onClick={handleSign(b.key)}>
                {loading ? <CircularProgress /> : "Sign"}
              </Button>
            </Row>
          );
        })}
      </Card>
    </CardContainer>
  );
};

export default AllBonds;
