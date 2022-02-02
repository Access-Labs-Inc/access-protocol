import React from "react";
import { useCentralState } from "../hooks/useCentralState";
import Card from "../components/Card";
import { styled } from "@mui/material/styles";

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
  width: 800,
  height: 350,
  display: "flex",
  justifyContent: "space-around",
  alignItems: "center",
  flexDirection: "column",
});

const CentralState = () => {
  const [centralState] = useCentralState();
  console.log(centralState);
  return (
    <Container>
      <CardContainer>
        <Card>
          <InnerCard>
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
            </div>
          </InnerCard>
        </Card>
      </CardContainer>
    </Container>
  );
};

export default CentralState;
