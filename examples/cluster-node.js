const http = require("http");
const port = process.env.PORT || 3000;

http.createServer((_req, res) => {
  res.end(`rspm worker ${process.env.NODE_APP_INSTANCE || "0"}\n`);
}).listen(port);

