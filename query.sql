SELECT DISTINCT
  "from"
FROM
  (
    SELECT
      "from"
    FROM
      erc20_ethereum.evt_transfer
    WHERE
      to = 0xf047ab4c75cebf0eb9ed34ae2c186f3611aeafa6 -- Zircuit Restaking Pool
    UNION
    SELECT
      to as "from"
    FROM
      erc20_ethereum.evt_transfer
    WHERE
      contract_address = 0x40357b9f22b4dff0bf56a90661b8ec106c259d29 -- YT USDe v1
      OR contract_address = 0x90c98ab215498b72abfec04c651e2e496ba364c0 -- LP USDe v1
      OR contract_address = 0x029aad400f6092dc735a65be95102efcc2fe64bf -- YT rsETH v1
      OR contract_address = 0x445d25a1c31445fb29e65d12da8e0eea38174176 -- LP rsETH v1
      OR contract_address = 0x7c2d26182adeef96976035986cf56474fec03bda -- YT weETH v1
      OR contract_address = 0xe26d7f9409581f606242300fbfe63f56789f2169 -- LP weETH v1
      OR contract_address = 0x98601e27d41ccff643da9d981dc708cf9ef1f150 -- YT ezETH v1
      OR contract_address = 0xd7e0809998693fd87e81d51de1619fd0ee658031 -- LP ezETH v1
      OR contract_address = 0x40357b9f22b4dff0bf56a90661b8ec106c259d29 -- YT USDe v2
      OR contract_address = 0x90c98ab215498b72abfec04c651e2e496ba364c0 -- LP USDe v2
      OR contract_address = 0x36bc05a1072ef7d763d5f11f463915aa1efb8ca8 -- YT rsETH v2
      OR contract_address = 0x99184849e35d91dd85f50993bbb03a42fc0a6fe7 -- LP rsETH v2
      OR contract_address = 0x323da63d354c9d79df927fd21ce5b97add3a50d9 -- YT weETH v2
      OR contract_address = 0x6c269dfc142259c52773430b3c78503cc994a93e -- LP weETH v2
      OR contract_address = 0x87baf4b42c075db7eb1932a0a49a5465e9a5ce9f -- YT ezETH v2
      OR contract_address = 0xee6bdfac6767efef0879b924fea12a3437d281a2 -- LP ezETH v2
  );
