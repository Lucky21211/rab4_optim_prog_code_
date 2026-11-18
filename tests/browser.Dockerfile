FROM mcr.microsoft.com/playwright:v1.51.1-noble
WORKDIR /tests
RUN npm init -y && npm install --save-exact playwright@1.51.1
COPY tests/capture.cjs ./capture.cjs
CMD ["node", "capture.cjs"]
