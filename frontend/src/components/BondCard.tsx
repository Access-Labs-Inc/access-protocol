import React, { useState } from "react";
import Card from "./Card";
import { styled } from "@mui/material/styles";
import { useBondInfo } from "../hooks/useBondInfo";
import { PublicKey } from "@solana/web3.js";
import { Button } from "@mui/material";
import {
  claimBond,
  claimBondRewards,
  ACCESS_PROGRAM_ID,
  unlockBondTokens,
} from "@access";
import { sendTx } from "../utils/send";
import CircularProgress from "@mui/material/CircularProgress";
import { useConnection, useWallet } from "@solana/wallet-adapter-react";
import {
  Token,
  ASSOCIATED_TOKEN_PROGRAM_ID,
  TOKEN_PROGRAM_ID,
} from "@solana/spl-token";
import { notify } from "../utils/notifications";
import { refreshAllCaches } from "../utils/fetch-loop";
import { useCentralState } from "../hooks/useCentralState";

const Title = styled("span")({
  fontSize: 25,
});

const Row = styled("div")({
  display: "flex",
  alignItems: "center",
  justifyContent: "space-between",
});

const Label = styled("span")({
  fontSize: 24,
  fontWeight: "bold",
});

const Value = styled("span")({
  fontSize: 24,
  opacity: 0.8,
});

const CardContainer = styled("div")({
  height: 600,
  width: "80%",
});

const BondCard = ({ bondKey }: { bondKey: string }) => {
  const [loading, setLoading] = useState(false);
  const [bondInfo] = useBondInfo(new PublicKey(bondKey));
  const { connection } = useConnection();
  const { publicKey, sendTransaction } = useWallet();
  const [centralState] = useCentralState();

  const handleClaimBond = async () => {
    if (!publicKey || !bondInfo) return;
    try {
      setLoading(true);
      // Get token source account i.e the account used to buy the bond
      // Assume it's an associated token account
      const quoteTokenSource = await Token.getAssociatedTokenAddress(
        ASSOCIATED_TOKEN_PROGRAM_ID,
        TOKEN_PROGRAM_ID,
        bondInfo.quoteMint,
        publicKey
      );

      const ix = await claimBond(
        connection,
        new PublicKey(bondKey),
        publicKey,
        quoteTokenSource,
        ACCESS_PROGRAM_ID
      );
      const tx = await sendTx(connection, publicKey, [ix], sendTransaction);
      console.log(tx);
      notify({
        message: "Success claiming bond",
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

  const handleClaimRewards = async () => {
    if (!publicKey || !centralState) return;
    try {
      setLoading(true);
      // Assume it's an associated token account
      const rewardsDestination = await Token.getAssociatedTokenAddress(
        ASSOCIATED_TOKEN_PROGRAM_ID,
        TOKEN_PROGRAM_ID,
        centralState.tokenMint,
        publicKey
      );
      const ix = await claimBondRewards(
        connection,
        new PublicKey(bondKey),
        rewardsDestination,
        ACCESS_PROGRAM_ID
      );
      const tx = await sendTx(connection, publicKey, [ix], sendTransaction);
      console.log(tx.value);
      notify({
        message: "Success claiming rewards",
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

  const handleUnlockTokens = async () => {
    if (!publicKey || !centralState) return;
    try {
      // Assume it's an associated token account
      const rewardsDestination = await Token.getAssociatedTokenAddress(
        ASSOCIATED_TOKEN_PROGRAM_ID,
        TOKEN_PROGRAM_ID,
        centralState.tokenMint,
        publicKey
      );
      const ix = await unlockBondTokens(
        connection,
        new PublicKey(bondKey),
        rewardsDestination,
        ACCESS_PROGRAM_ID
      );
      const tx = await sendTx(connection, publicKey, [ix], sendTransaction);
      console.log(tx.value);
      notify({
        message: "Success claiming rewards",
        variant: "success",
      });
    } catch (err) {
      console.log(err);
      // @ts-ignore
      notify({ message: `Error ${err.message}`, variant: "error" });
    } finally {
    }
  };

  return (
    <CardContainer>
      <Card>
        <Title>Bond info</Title>
        <div style={{ width: "100%" }}>
          <Row>
            <Label>Owner</Label>
            <Value>{bondInfo?.owner.toBase58()}</Value>
          </Row>
          <Row>
            <Label>Total sold</Label>
            <Value>{bondInfo?.totalAmountSold.toString()}</Value>
          </Row>
          <Row>
            <Label>Total staked</Label>
            <Value>{bondInfo?.totalStaked.toString()}</Value>
          </Row>
          <Row>
            <Label>Total quote amount</Label>
            <Value>{bondInfo?.totalQuoteAmount.toString()}</Value>
          </Row>
          <Row>
            <Label>Quote mint</Label>
            <Value>{bondInfo?.quoteMint.toBase58()}</Value>
          </Row>
          <Row>
            <Label>Seller token account</Label>
            <Value>{bondInfo?.sellerTokenAccount.toBase58()}</Value>
          </Row>
          <Row>
            <Label>Unlock start date</Label>
            <Value>{bondInfo?.unlockStartDate.toString()}</Value>
          </Row>
          <Row>
            <Label>Unlock period</Label>
            <Value>{bondInfo?.unlockPeriod.toString()}</Value>
          </Row>
          <Row>
            <Label>Unlock amount</Label>
            <Value>{bondInfo?.unlockAmount.toString()}</Value>
          </Row>
          <Row>
            <Label>Last unlock time</Label>
            <Value>{bondInfo?.lastUnlockTime.toString()}</Value>
          </Row>
          <Row>
            <Label>Total unlocked amount</Label>
            <Value>{bondInfo?.totalUnlockedAmount.toString()}</Value>
          </Row>
          <Row>
            <Label>Pool minimum at creation</Label>
            <Value>{bondInfo?.poolMinimumAtCreation.toString()}</Value>
          </Row>
          <Row>
            <Label>Stake pool</Label>
            <Value>{bondInfo?.stakePool.toBase58()}</Value>
          </Row>
          <Row>
            <Label>Last claimed time</Label>
            <Value>{bondInfo?.lastClaimedTime.toString()}</Value>
          </Row>
          {bondInfo?.sellers.map((e, idx) => {
            return (
              <Row>
                <Label>Seller {idx}</Label>
                <Value>{e.toBase58()}</Value>
              </Row>
            );
          })}
        </div>
        <Button onClick={handleClaimBond} variant="contained">
          {loading ? <CircularProgress color="inherit" /> : "Claim bond"}
        </Button>
        <Button onClick={handleClaimRewards} variant="contained">
          {loading ? <CircularProgress color="inherit" /> : "Claim rewards"}
        </Button>
        <Button onClick={handleUnlockTokens} variant="contained">
          {loading ? <CircularProgress color="inherit" /> : "Unlock tokens"}
        </Button>
      </Card>
    </CardContainer>
  );
};

export default BondCard;
