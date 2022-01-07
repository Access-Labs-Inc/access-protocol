FROM golang:latest

WORKDIR /usr/src/access-protocol/backends/go

COPY . .

RUN go build -o access_backend .

CMD [ "./access_backend" ]