FROM python:3.12-alpine
WORKDIR /app
COPY requirements.txt ./
RUN pip install --no-cache-dir -r requirements.txt
COPY legacy.py ./
COPY entrypoint.sh /entrypoint.sh
RUN chmod +x /entrypoint.sh
ENV TZ=UTC PYTHONUNBUFFERED=1
ENTRYPOINT ["/entrypoint.sh"]
