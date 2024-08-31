exports.handler = async (params) => {
  // read from DB here
  return {
    statusCode: 200,
    body: JSON.stringify({
      sum:
        Number(params.queryStringParameters.a) +
        Number(params.queryStringParameters.b),
    }),
  };
};
