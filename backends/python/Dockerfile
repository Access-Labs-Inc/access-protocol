FROM python:latest

WORKDIR /usr/src/the-block/backends/python

COPY requirements.txt .

RUN pip3 install -r requirements.txt

COPY src/ .

CMD [ "python3", "-m" , "flask", "run", "--host=0.0.0.0", "--port=3001"]