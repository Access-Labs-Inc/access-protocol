import React from "react";
import { useCentralState } from "../hooks/useCentralState";
import Card from "./Card";
import { styled } from "@mui/material/styles";
import { useTotalSupply } from "../hooks/useTotalSupply";

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
  height: 200,
});

const CentralState = () => {
  const [centralState] = useCentralState();
  const [totalSupply] = useTotalSupply(centralState?.tokenMint);
  return (
    <CardContainer>
      <Card>
        <Title>Central state info</Title>
        <div style={{ width: "100%" }}>
          <Row>
            <Label>Authority</Label>
            <Value>{centralState?.authority.toBase58()}</Value>
          </Row>
          <Row>
            <Label>Current inflation</Label>
            <Value>{centralState?.dailyInflation.toNumber()}</Value>
          </Row>
          <Row>
            <Label>ACCESS Mint</Label>
            <Value>{centralState?.tokenMint.toBase58()}</Value>
          </Row>
          <Row>
            <Label>Total supply</Label>
            <Value>{totalSupply}</Value>
          </Row>
        </div>
      </Card>
    </CardContainer>
  );
};

export default CentralState;
