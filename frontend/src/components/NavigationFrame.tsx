import React from "react";
import TopBar from "./TopBar";

const NavigationFrame = ({ children }: { children: React.ReactNode }) => {
  return (
    <>
      <TopBar />
      <div
        style={{
          flexGrow: 1,
          overflowX: "hidden",
          overflowY: "hidden",
          width: "100%",
        }}
      >
        {children}
      </div>
    </>
  );
};

export default NavigationFrame;
