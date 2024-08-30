exports.handler = async (params) => {
  // read from DB here
  return {
    statusCode: 200,
    body: JSON.stringify({
      "hello from": "the other endpoint =)"
    })
  };
};
