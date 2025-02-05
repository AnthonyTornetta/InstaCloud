exports.handler = async (params) => {
  // read from DB here
  if (!params.queryStringParameters) {
    return {
      statusCode: 400,
      body: JSON.stringify({
        error: "Requires a and b parameters",
      }),
    };
  }

  return {
    statusCode: 200,
    body: JSON.stringify({
      sum:
        Number(params.queryStringParameters.a) +
        Number(params.queryStringParameters.b),
    }),
  };
};
