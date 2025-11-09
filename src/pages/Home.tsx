import React from "react";
import { Layout, Text } from "@stellar/design-system";
// import { GuessTheNumber } from "../components/GuessTheNumber";

const Home: React.FC = () => (
  <Layout.Content>
    <Layout.Inset>
      <Text as="h1" size="xl">
        Welcome to your cylo!
      </Text>
      <Text as="p" size="md">
        Your most trusted platform for farm to table delivery. Buy fresh, buy
        cheap🍅.
      </Text>
    </Layout.Inset>
  </Layout.Content>
);

export default Home;

// # Use it
// nvm use 22.12.0

// # Set it as default
// nvm alias default 22.12.0
