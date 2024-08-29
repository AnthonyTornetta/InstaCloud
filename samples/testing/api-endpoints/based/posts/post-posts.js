exports.handler = async (params) => {
  return {
    responseCode: 200,
    body: JSON.stringify(params)
  }
};
