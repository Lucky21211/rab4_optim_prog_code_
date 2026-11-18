FROM python:3.12-slim
RUN pip install --no-cache-dir psycopg[binary]==3.2.6
WORKDIR /work
COPY . .
CMD ["python", "-m", "unittest", "discover", "-s", "tests", "-v"]
