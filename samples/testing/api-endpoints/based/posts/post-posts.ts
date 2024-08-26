interface Body {
  postContent: string;
}

export default async (params: Body) => {
  console.log("Write to DB: " + params);
};
